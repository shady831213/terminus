use std::marker::PhantomData;
use std::convert::TryFrom;
use crate::processor::trap::Exception;
use terminus_global::RegT;
use std::rc::Rc;
use std::sync::Arc;
use crate::processor::{ProcessorState, Privilege};
use crate::processor::extensions::i::csrs::*;
use crate::processor::extensions::s::csrs::*;
use terminus_macros::*;
use crate::devices::bus::Bus;

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
    p: Rc<ProcessorState>,
    bus: Arc<Bus>,
    fetch_tlb: RefCell<TLB>,
    load_tlb: RefCell<TLB>,
    store_tlb: RefCell<TLB>,
}

impl Mmu {
    pub fn new(p: &Rc<ProcessorState>, bus: &Arc<Bus>) -> Mmu {
        Mmu {
            p: p.clone(),
            bus: bus.clone(),
            fetch_tlb: RefCell::new(TLB::new(256)),
            load_tlb: RefCell::new(TLB::new(256)),
            store_tlb: RefCell::new(TLB::new(256)),
        }
    }
    #[inline(always)]
    fn pmpcfgs_iter(&self) -> PmpCfgsIter {
        PmpCfgsIter::new(self, PhantomData)
    }
    #[inline(always)]
    fn get_pmpaddr(&self, idx: u8) -> RegT {
        self.p.csrs::<ICsrs>().unwrap().read(0x3b0 + idx as RegT).unwrap()
    }
    fn match_pmpcfg_entry(&self, addr: u64, len: usize) -> Option<PmpCfgEntry> {
        self.pmpcfgs_iter().enumerate()
            .find(|(idx, entry)| {
                ((addr >> 2)..((addr + len as u64 - 1) >> 2) + 1)
                    .map(|trail_addr| {
                        match PmpAType::try_from(entry.a()).unwrap() {
                            PmpAType::OFF => false,
                            PmpAType::TOR => {
                                let low = if *idx == 0 {
                                    0
                                } else {
                                    self.get_pmpaddr((*idx - 1) as u8)
                                };
                                let high = self.get_pmpaddr(*idx as u8);
                                trail_addr >= low && trail_addr < high
                            }
                            PmpAType::NA4 => {
                                let pmpaddr = self.get_pmpaddr(*idx as u8);
                                trail_addr == pmpaddr
                            }
                            PmpAType::NAPOT => {
                                let pmpaddr = self.get_pmpaddr(*idx as u8);
                                let trialing_ones = (!pmpaddr).trailing_zeros();
                                (trail_addr >> trialing_ones) == (pmpaddr >> trialing_ones)
                            }
                        }
                    })
                    .fold(true, |acc, m| { acc && m })
            })
            .map(|(_, entry)| { entry })
    }
    #[inline(always)]
    fn check_pmp(&self, addr: u64, len: usize, opt: MmuOpt, privilege: Privilege) -> bool {
        if let Some(entry) = self.match_pmpcfg_entry(addr, len) {
            privilege == Privilege::M && entry.l() == 0 || opt.pmp_match(&entry)
        } else {
            privilege == Privilege::M
        }
    }
    #[inline(always)]
    fn pte_info(&self) -> PteInfo {
        PteInfo::new(self.p.csrs::<SCsrs>().unwrap().satp().deref())
    }
    #[inline(always)]
    fn get_privileage(&self, opt: MmuOpt) -> Privilege {
        if self.p.csrs::<ICsrs>().unwrap().mstatus().mprv() == 1 && opt != MmuOpt::Fetch {
            Privilege::try_from(self.p.csrs::<ICsrs>().unwrap().mstatus().mpp() as u8).unwrap()
        } else {
            self.p.privilege.borrow().clone()
        }
    }

    fn check_pte_privilege(&self, addr: RegT, pte_attr: &PteAttr, opt: &MmuOpt, privilege: &Privilege) -> Result<(), Exception> {
        let priv_s = *privilege == Privilege::S;
        let pte_x = pte_attr.x() == 1;
        let pte_u = pte_attr.u() == 1;
        let pte_r = pte_attr.r() == 1;
        let pte_w = pte_attr.w() == 1;
        let mxr = self.p.csrs::<ICsrs>().unwrap().mstatus().mxr() == 1;
        let sum = self.p.csrs::<ICsrs>().unwrap().mstatus().sum() == 1;
        match opt {
            MmuOpt::Fetch => {
                if !pte_x || pte_u == priv_s {
                    return Err(opt.pagefault_exception(addr));
                }
            }
            MmuOpt::Load => {
                if priv_s && !sum && pte_u || !pte_u && !priv_s || !pte_r && !mxr || mxr && !pte_r && !pte_x {
                    return Err(opt.pagefault_exception(addr));
                }
            }
            MmuOpt::Store => {
                if priv_s && !sum && pte_u || !pte_u && !priv_s || !pte_w || !pte_r {
                    return Err(opt.pagefault_exception(addr));
                }
            }
        }
        Ok(())
    }

    fn pt_walk(&self, vaddr: &Vaddr, opt: MmuOpt, privilege: Privilege) -> Result<u64, Exception> {
        let info = self.pte_info();
        //step 1
        let ppn = self.p.csrs::<SCsrs>().unwrap().satp().ppn();
        let mut a = ppn * info.page_size as RegT;
        let mut level = info.level - 1;
        let mut leaf_pte: Pte;
        let mut pte_addr: u64;
        loop {
            //step 2
            pte_addr = (a + vaddr.vpn(level).unwrap() * info.size as RegT) as u64;
            if !self.check_pmp(pte_addr, info.size, MmuOpt::Load, Privilege::S) {
                return Err(opt.access_exception(vaddr.value()));
            }
            let pte = match Pte::load(&info, self.bus.deref(), pte_addr) {
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
                a = pte.ppn_all() * info.page_size as RegT;
            }
        }
        //step 5
        self.check_pte_privilege(vaddr.value(), &leaf_pte.attr(), &opt, &privilege)?;
        //step 6
        for l in 0..level {
            if leaf_pte.ppn(l).unwrap() != 0 {
                return Err(opt.pagefault_exception(vaddr.value()));
            }
        }
        //step 7
        if leaf_pte.attr().d() == 0 && opt == MmuOpt::Store || leaf_pte.attr().a() == 0 {
            if self.p.config.enable_dirty {
                let mut new_attr = leaf_pte.attr();
                new_attr.set_a(1);
                new_attr.set_d((opt == MmuOpt::Store) as u8);
                if !self.check_pmp(pte_addr, info.size, MmuOpt::Store, Privilege::S) {
                    return Err(opt.access_exception(vaddr.value()));
                }
                leaf_pte.set_attr(new_attr);
                if leaf_pte.store(self.bus.deref(), pte_addr).is_err() {
                    return Err(opt.access_exception(vaddr.value()));
                }
            } else {
                return Err(opt.pagefault_exception(vaddr.value()));
            }
        }
        //step 8
        Ok(Paddr::new(vaddr, &leaf_pte, &info, level).value() as u64)
    }

    pub fn flush_tlb(&self) {
        self.fetch_tlb.borrow_mut().invalid_all();
        self.load_tlb.borrow_mut().invalid_all();
        self.store_tlb.borrow_mut().invalid_all();
    }

    pub fn translate(&self, va: RegT, len: RegT, opt: MmuOpt) -> Result<u64, Exception> {
        let privilege = self.get_privileage(opt);
        if privilege == Privilege::M {
            return Ok(va as u64);
        }
        if self.pte_info().mode == PteMode::Bare {
            return Ok(va as u64);
        }
        let vaddr = Vaddr::new(&self.pte_info().mode, va);
        let mut tlb = match opt {
            MmuOpt::Fetch => self.fetch_tlb.borrow_mut(),
            MmuOpt::Load => self.load_tlb.borrow_mut(),
            MmuOpt::Store => self.store_tlb.borrow_mut(),
        };
        if let Some(ppn) = tlb.get_ppn(vaddr.vpn_all()) {
            let pa = ppn << 12 | vaddr.offset();
            return Ok(pa);
        }
        match self.pt_walk(&vaddr, opt, privilege) {
            Ok(pa) => if !self.check_pmp(pa, len as usize, opt, privilege) {
                return Err(opt.access_exception(va));
            } else {
                tlb.set_entry(vaddr.vpn_all(), pa >> 12);
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
    assert_eq!(p.mmu().match_pmpcfg_entry(0, 1), None);
    //NA4
    p.state.csrs::<ICsrs>().unwrap().pmpcfg0_mut().set_bit_range(4, 3, PmpAType::NA4.into());
    p.state.csrs::<ICsrs>().unwrap().pmpaddr0_mut().set(0x8000_0000 >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x8000_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x8000_0000, 5).is_none());

    //NAPOT
    p.state.csrs::<ICsrs>().unwrap().pmpcfg3_mut().set_bit_range(4, 3, PmpAType::NAPOT.into());
    p.state.csrs::<ICsrs>().unwrap().pmpaddr12_mut().set((0x2000_0000 + 0x1_0000 - 1) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x2000_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2000_ffff, 1).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2000_ffff, 2).is_none());
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2000_ffff, 1), p.mmu().match_pmpcfg_entry(0x2000_0000, 4));
    assert_eq!(p.mmu().match_pmpcfg_entry(0x1000_ffff, 1), None);
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2001_0000, 4), None);
    //TOR
    p.state.csrs::<ICsrs>().unwrap().pmpcfg3_mut().set_bit_range(12, 11, PmpAType::TOR.into());
    p.state.csrs::<ICsrs>().unwrap().pmpaddr13_mut().set((0x2000_0000 + 0x1_0000) >> 2);
    p.state.csrs::<ICsrs>().unwrap().pmpcfg3_mut().set_bit_range(20, 19, PmpAType::TOR.into());
    p.state.csrs::<ICsrs>().unwrap().pmpaddr14_mut().set((0x2000_0000 + 0x2_0000) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x2001_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2001_ffff, 1).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2001_ffff, 2).is_none());
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2002_0000, 4), None);
    p.state.csrs::<ICsrs>().unwrap().pmpcfg3_mut().set_bit_range(23, 23, 1);
    assert!(p.mmu().match_pmpcfg_entry(0x2001_0000, 4).is_some());
}