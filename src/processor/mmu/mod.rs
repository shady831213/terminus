use std::marker::PhantomData;
use std::convert::TryFrom;
use crate::processor::trap::Exception;
use terminus_global::{RegT, InsnT};
use std::rc::Rc;
use std::sync::Arc;
use crate::processor::ProcessorState;
use terminus_macros::*;
use crate::devices::bus::Bus;
use crate::processor::extensions::i::csrs::ICsrs;

mod pmp;

use pmp::*;

mod pte;

use pte::*;

mod tlb;

use tlb::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MmuOpt {
    Load,
    Store,
    Fetch,
}

impl MmuOpt {
    fn access_exception(&self, addr: RegT) -> Exception {
        match self {
            MmuOpt::Fetch => Exception::FetchAccess(addr as u64),
            MmuOpt::Load => Exception::LoadAccess(addr as u64),
            MmuOpt::Store => Exception::StoreAccess(addr as u64)
        }
    }

    fn pagefault_exception(&self, addr: RegT) -> Exception {
        match self {
            MmuOpt::Fetch => Exception::FetchPageFault(addr as u64),
            MmuOpt::Load => Exception::LoadPageFault(addr as u64),
            MmuOpt::Store => Exception::StorePageFault(addr as u64)
        }
    }

    fn pmp_match(&self, pmpcfg: &PmpCfgEntry) -> bool {
        match self {
            MmuOpt::Fetch => pmpcfg.x() == 1,
            MmuOpt::Load => pmpcfg.r() == 1,
            MmuOpt::Store => pmpcfg.w() == 1
        }
    }
}

pub struct Mmu {
    bus: Arc<Bus>,
    fetch_tlb: RefCell<TLB>,
    load_tlb: RefCell<TLB>,
    store_tlb: RefCell<TLB>,
}

impl Mmu {
    pub fn new(bus: &Arc<Bus>) -> Mmu {
        Mmu {
            bus: bus.clone(),
            fetch_tlb: RefCell::new(TLB::new()),
            load_tlb: RefCell::new(TLB::new()),
            store_tlb: RefCell::new(TLB::new()),
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn pmpcfgs_iter(&self, state: &ProcessorState) -> PmpCfgsIter {
        PmpCfgsIter::new(state.icsrs(), PhantomData)
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn match_pmpcfg_entry(&self, state: &ProcessorState, addr: u64, len: usize) -> Option<PmpCfgEntry> {
        self.pmpcfgs_iter(state).enumerate()
            .find(|(idx, entry)| {
                ((addr >> 2)..((addr + len as u64 - 1) >> 2) + 1)
                    .map(|trail_addr| {
                        match PmpAType::try_from(entry.a()).unwrap() {
                            PmpAType::OFF => false,
                            PmpAType::TOR => {
                                let low = if *idx == 0 {
                                    0
                                } else {
                                    state.icsrs().read(0x3b0 + ((*idx - 1) as u8 as InsnT)).unwrap()
                                };
                                let high = state.icsrs().read(0x3b0 + (*idx as u8 as InsnT)).unwrap();
                                trail_addr >= low && trail_addr < high
                            }
                            PmpAType::NA4 => {
                                let pmpaddr = state.icsrs().read(0x3b0 + (*idx as u8 as InsnT)).unwrap();
                                trail_addr == pmpaddr
                            }
                            PmpAType::NAPOT => {
                                let pmpaddr = state.icsrs().read(0x3b0 + (*idx as u8 as InsnT)).unwrap();
                                let trialing_ones = (!pmpaddr).trailing_zeros();
                                (trail_addr >> trialing_ones) == (pmpaddr >> trialing_ones)
                            }
                        }
                    })
                    .fold(true, |acc, m| { acc && m })
            })
            .map(|(_, entry)| { entry })
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn check_pmp(&self, state: &ProcessorState, addr: u64, len: usize, opt: &MmuOpt, privilege: &u8) -> bool {
        if let Some(entry) = self.match_pmpcfg_entry(state, addr, len) {
            *privilege == 3 && entry.l() == 0 || opt.pmp_match(&entry)
        } else {
            *privilege == 3
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn get_privileage(&self, state: &ProcessorState, opt: &MmuOpt) -> u8 {
        let is_mprv = state.icsrs().mstatus().mprv() == 1;
        let mpp = state.icsrs().mstatus().mpp() as u8 & 3;
        match opt {
            &MmuOpt::Load if is_mprv => mpp,
            &MmuOpt::Store if is_mprv => mpp,
            _ => state.privilege().into()
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn check_pte_privilege(&self,state:&ProcessorState, addr: RegT, pte_attr: &PteAttr, opt: &MmuOpt, privilege: &u8) -> Result<(), Exception> {
        let priv_s = *privilege == 1;
        let pte_x = pte_attr.x() == 1;
        let pte_u = pte_attr.u() == 1;
        let pte_r = pte_attr.r() == 1;
        let pte_w = pte_attr.w() == 1;
        let mxr = state.icsrs().mstatus().mxr() == 1;
        let sum = state.icsrs().mstatus().sum() == 1;
        match opt {
            &MmuOpt::Fetch => {
                if !pte_x || pte_u == priv_s {
                    return Err(opt.pagefault_exception(addr));
                }
            }
            &MmuOpt::Load => {
                if priv_s && !sum && pte_u || !pte_u && !priv_s || !pte_r && !mxr || mxr && !pte_r && !pte_x {
                    return Err(opt.pagefault_exception(addr));
                }
            }
            &MmuOpt::Store => {
                if priv_s && !sum && pte_u || !pte_u && !priv_s || !pte_w || !pte_r {
                    return Err(opt.pagefault_exception(addr));
                }
            }
        }
        Ok(())
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn pt_walk(&self, state: &ProcessorState, vaddr: &Vaddr, opt: &MmuOpt, privilege: &u8, info: &PteInfo) -> Result<u64, Exception> {
        //step 1
        let ppn = state.scsrs().satp().ppn();
        let mut a = (ppn << info.page_size_shift) as RegT;
        let mut level = info.level - 1;
        let mut leaf_pte: Pte;
        let mut pte_addr: u64;
        loop {
            //step 2
            pte_addr = (a + (vaddr.vpn(level) << (info.size_shift as RegT))) as u64;
            if !self.check_pmp(state, pte_addr, 1 << info.size_shift, &MmuOpt::Load, &1) {
                return Err(opt.access_exception(vaddr.value()));
            }
            let pte = match Pte::load(info, self.bus.deref(), pte_addr) {
                Ok(pte) => pte,
                Err(_) => return Err(opt.access_exception(vaddr.value()))
            };
            //step 3
            if pte.attr().v() == 0 || pte.attr().r() == 0 && pte.attr().w() == 1 {
                return Err(opt.pagefault_exception(vaddr.value()));
            }
            //step 4
            if pte.attr().r() == 1 || pte.attr().x() == 1 {
                leaf_pte = pte;
                break;
            } else if level == 0 {
                return Err(opt.pagefault_exception(vaddr.value()));
            } else {
                level -= 1;
                a = pte.ppn_all() << info.page_size_shift as RegT;
            }
        }
        //step 5
        self.check_pte_privilege(state,vaddr.value(), &leaf_pte.attr(), opt, privilege)?;
        //step 6
        for l in 0..level {
            if leaf_pte.ppn(l) != 0 {
                return Err(opt.pagefault_exception(vaddr.value()));
            }
        }
        //step 7
        if leaf_pte.attr().d() == 0 && *opt == MmuOpt::Store || leaf_pte.attr().a() == 0 {
            if state.config().enable_dirty {
                let mut new_attr = leaf_pte.attr();
                new_attr.set_a(1);
                new_attr.set_d((*opt == MmuOpt::Store) as u8);
                if !self.check_pmp(state, pte_addr, 1 << info.size_shift, &MmuOpt::Store, &1) {
                    return Err(opt.access_exception(vaddr.value()));
                }
                leaf_pte.set_attr(&new_attr);
                if leaf_pte.store(self.bus.deref(), pte_addr).is_err() {
                    return Err(opt.access_exception(vaddr.value()));
                }
            } else {
                return Err(opt.pagefault_exception(vaddr.value()));
            }
        }
        //step 8
        Ok(Paddr::new(vaddr, &leaf_pte, info, level).value() as u64)
    }

    pub fn flush_tlb(&self) {
        self.fetch_tlb.borrow_mut().invalid_all();
        self.load_tlb.borrow_mut().invalid_all();
        self.store_tlb.borrow_mut().invalid_all();
    }

    pub fn translate(&self, state: &ProcessorState, va: RegT, len: RegT, opt: MmuOpt) -> Result<u64, Exception> {
        let privilege = self.get_privileage(state, &opt);
        if privilege == 3 {
            return Ok(va as u64);
        }
        let info = PteInfo::new(state.scsrs().satp().deref());
        if info.mode == PTE_BARE {
            return Ok(va as u64);
        }
        let vaddr = Vaddr::new(info.mode, va);
        let mut tlb = match opt {
            MmuOpt::Fetch => self.fetch_tlb.borrow_mut(),
            MmuOpt::Load => self.load_tlb.borrow_mut(),
            MmuOpt::Store => self.store_tlb.borrow_mut(),
        };
        if let Some(ppn) = tlb.get_ppn(vaddr.vpn_all()) {
            let pa = (ppn << (info.page_size_shift as u64)) | vaddr.offset();
            return Ok(pa);
        }
        match self.pt_walk(state, &vaddr, &opt, &privilege, &info) {
            Ok(pa) => if !self.check_pmp(state, pa, len as usize, &opt, &privilege) {
                return Err(opt.access_exception(va));
            } else {
                tlb.set_entry(vaddr.vpn_all(), pa >> (info.page_size_shift as u64));
                Ok(pa)
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
use terminus_global::XLen;
#[cfg(test)]
use crate::processor::ProcessorCfg;
#[cfg(test)]
use crate::system::System;
use std::cell::RefCell;

#[test]
fn pmp_basic_test() {
    let sys = System::new("test", "top_tests/elf/rv64ui-p-add", vec![ProcessorCfg {
        xlen: XLen::X32,
        enable_dirty: true,
        extensions: vec![].into_boxed_slice(),
        freq: 1000000000,
    }], 100);
    sys.reset(vec![-1i64 as u64]).unwrap();

    let p = sys.processor(0).unwrap();
    //no valid region
    assert_eq!(p.mmu().match_pmpcfg_entry(p.state(), 0, 1), None);
    //NA4
    p.state().icsrs().pmpcfg0_mut().set_bit_range(4, 3, PmpAType::NA4.into());
    p.state().icsrs().pmpaddr0_mut().set(0x8000_0000 >> 2);
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x8000_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x8000_0000, 5).is_none());

    //NAPOT
    p.state().icsrs().pmpcfg3_mut().set_bit_range(4, 3, PmpAType::NAPOT.into());
    p.state().icsrs().pmpaddr12_mut().set((0x2000_0000 + 0x1_0000 - 1) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x2000_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x2000_ffff, 1).is_some());
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x2000_ffff, 2).is_none());
    assert_eq!(p.mmu().match_pmpcfg_entry(p.state(), 0x2000_ffff, 1), p.mmu().match_pmpcfg_entry(p.state(), 0x2000_0000, 4));
    assert_eq!(p.mmu().match_pmpcfg_entry(p.state(), 0x1000_ffff, 1), None);
    assert_eq!(p.mmu().match_pmpcfg_entry(p.state(), 0x2001_0000, 4), None);
    //TOR
    p.state().icsrs().pmpcfg3_mut().set_bit_range(12, 11, PmpAType::TOR.into());
    p.state().icsrs().pmpaddr13_mut().set((0x2000_0000 + 0x1_0000) >> 2);
    p.state().icsrs().pmpcfg3_mut().set_bit_range(20, 19, PmpAType::TOR.into());
    p.state().icsrs().pmpaddr14_mut().set((0x2000_0000 + 0x2_0000) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x2001_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x2001_ffff, 1).is_some());
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x2001_ffff, 2).is_none());
    assert_eq!(p.mmu().match_pmpcfg_entry(p.state(), 0x2002_0000, 4), None);
    p.state().icsrs().pmpcfg3_mut().set_bit_range(23, 23, 1);
    assert!(p.mmu().match_pmpcfg_entry(p.state(), 0x2001_0000, 4).is_some());
}