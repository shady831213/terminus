use super::*;
use crate::Exception;
use std::marker::PhantomData;
use terminus_spaceport::memory::region::{U32Access, U64Access};
use std::convert::TryFrom;

mod pmp;
use pmp::*;

mod pte;
use pte::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MmuOpt {
    Load,
    Store,
    Fetch,
}

pub struct Mmu {
    p: Rc<ProcessorState>,
}

impl Mmu {
    pub fn new(p: &Rc<ProcessorState>) -> Mmu {
        Mmu {
            p: p.clone(),
        }
    }

    fn pmpcfgs_iter(&self) -> PmpCfgsIter {
        PmpCfgsIter::new(self, PhantomData)
    }

    fn get_pmpaddr(&self, idx: u8) -> RegT {
        self.p.csr().read(0x3b0 + idx as RegT).unwrap()
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

    pub fn check_pmp(&self, addr: u64, len: usize, opt: MmuOpt, privilege: Privilege) -> bool {
        if let Some(entry) = self.match_pmpcfg_entry(addr, len) {
            privilege == Privilege::M && entry.l() == 0 ||
                opt == MmuOpt::Fetch && entry.x() == 1 ||
                opt == MmuOpt::Load && entry.r() == 1 ||
                opt == MmuOpt::Store && entry.w() == 1
        } else {
            privilege == Privilege::M
        }
    }

    fn pte_info(&self) -> PteInfo {
        PteInfo::new(&self.p.csr().satp)
    }

    fn get_privileage(&self, opt: MmuOpt) -> Privilege {
        if self.p.csr().mstatus.mprv() == 1 && opt != MmuOpt::Fetch {
            Privilege::try_from(self.p.csr().mstatus.mpp() as u8).unwrap()
        } else {
            self.p.privilege.borrow().clone()
        }
    }

    pub fn va2pa(&self, va: RegT, len: RegT, opt: MmuOpt) -> Result<u64, Exception> {
        let privilege = self.get_privileage(opt);
        if privilege == Privilege::M {
            return Ok(va as u64);
        }
        let info = self.pte_info();
        if let Some(vaddr) = Vaddr::new(&info.mode, va) {
            //step 1
            let ppn = self.p.csr().satp.ppn();
            let mut a = ppn * info.page_size as RegT;
            let mut level = info.level - 1;
            let mut leaf_pte: Pte;
            let mut pte_addr: u64;
            loop {
                //step 2
                pte_addr = (a + vaddr.vpn(level).unwrap() * info.size as RegT) as u64;
                if !self.check_pmp(pte_addr, info.size, MmuOpt::Load, Privilege::S) {
                    return match opt {
                        MmuOpt::Fetch => Err(Exception::FetchAccess(va as u64)),
                        MmuOpt::Load => Err(Exception::LoadAccess(va as u64)),
                        MmuOpt::Store => Err(Exception::StoreAccess(va as u64))
                    };
                }
                let pte = Pte::new(&info.mode, match info.size {
                    4 => {
                        match U32Access::read(&self.p.bus, pte_addr) {
                            Ok(pte) => pte as RegT,
                            Err(_) => return match opt {
                                MmuOpt::Fetch => Err(Exception::FetchAccess(va as u64)),
                                MmuOpt::Load => Err(Exception::LoadAccess(va as u64)),
                                MmuOpt::Store => Err(Exception::StoreAccess(va as u64))
                            }
                        }
                    }
                    8 => {
                        match U64Access::read(&self.p.bus, pte_addr) {
                            Ok(pte) => pte as RegT,
                            Err(_) => return match opt {
                                MmuOpt::Fetch => Err(Exception::FetchAccess(va as u64)),
                                MmuOpt::Load => Err(Exception::LoadAccess(va as u64)),
                                MmuOpt::Store => Err(Exception::StoreAccess(va as u64))
                            }
                        }
                    }
                    _ => unreachable!()
                }).unwrap();

                //step 3
                if pte.attr().v() == 0 || pte.attr().r() == 0 && pte.attr().w() == 1 {
                    return match opt {
                        MmuOpt::Fetch => Err(Exception::FetchPageFault(va as u64)),
                        MmuOpt::Load => Err(Exception::LoadPageFault(va as u64)),
                        MmuOpt::Store => Err(Exception::StorePageFault(va as u64))
                    };
                }

                //step 4
                if pte.attr().r() == 1 || pte.attr().x() == 1 {
                    leaf_pte = pte;
                    break;
                } else if level == 0 {
                    return match opt {
                        MmuOpt::Fetch => Err(Exception::FetchPageFault(va as u64)),
                        MmuOpt::Load => Err(Exception::LoadPageFault(va as u64)),
                        MmuOpt::Store => Err(Exception::StorePageFault(va as u64))
                    };
                } else {
                    level -= 1;
                    a = pte.ppn_all() * info.page_size as RegT;
                }
            }
            //step 5
            match opt {
                MmuOpt::Fetch => {
                    if leaf_pte.attr().x() == 0 {
                        return Err(Exception::FetchPageFault(va as u64));
                    }
                    if leaf_pte.attr().u() == 0 && privilege != Privilege::S {
                        return Err(Exception::FetchPageFault(va as u64));
                    }
                    if leaf_pte.attr().u() == 1 && privilege == Privilege::S {
                        return Err(Exception::FetchPageFault(va as u64));
                    }
                }
                MmuOpt::Load => {
                    if privilege == Privilege::S && self.p.csr().mstatus.sum() == 0 && leaf_pte.attr().u() == 1 {
                        return Err(Exception::LoadPageFault(va as u64));
                    }
                    if leaf_pte.attr().u() == 0 && privilege != Privilege::S {
                        return Err(Exception::LoadPageFault(va as u64));
                    }
                    if self.p.csr().mstatus.mxr() == 0 && leaf_pte.attr().r() == 0 || self.p.csr().mstatus.mxr() == 1 && leaf_pte.attr().r() == 0 && leaf_pte.attr().x() == 0 {
                        return Err(Exception::LoadPageFault(va as u64));
                    }
                }
                MmuOpt::Store => {
                    if privilege == Privilege::S && self.p.csr().mstatus.sum() == 0 && leaf_pte.attr().u() == 1 {
                        return Err(Exception::StorePageFault(va as u64));
                    }
                    if leaf_pte.attr().u() == 0 && privilege != Privilege::S {
                        return Err(Exception::StorePageFault(va as u64));
                    }
                    if leaf_pte.attr().w() == 0 || leaf_pte.attr().r() == 0 {
                        return Err(Exception::StorePageFault(va as u64));
                    }
                }
            }
            //step 6
            if level > 0 && leaf_pte.ppn(level - 1).unwrap() != 0 {
                return match opt {
                    MmuOpt::Fetch => Err(Exception::FetchPageFault(va as u64)),
                    MmuOpt::Load => Err(Exception::LoadPageFault(va as u64)),
                    MmuOpt::Store => Err(Exception::StorePageFault(va as u64))
                };
            }

            //step 7
            if leaf_pte.attr().d() == 0 && opt == MmuOpt::Store || leaf_pte.attr().a() == 0 {
                if self.p.config.enabel_dirty {
                    let mut new_attr = leaf_pte.attr();
                    new_attr.set_a(1);
                    new_attr.set_d((opt == MmuOpt::Store) as u8);
                    if !self.check_pmp(pte_addr, info.size, MmuOpt::Store, Privilege::S) {
                        return match opt {
                            MmuOpt::Fetch => Err(Exception::FetchAccess(va as u64)),
                            MmuOpt::Load => Err(Exception::LoadAccess(va as u64)),
                            MmuOpt::Store => Err(Exception::StoreAccess(va as u64))
                        };
                    }
                    leaf_pte.set_attr(new_attr);
                    match info.size {
                        4 => {
                            match U32Access::write(&self.p.bus, pte_addr, leaf_pte.value() as u32) {
                                Ok(_) => {}
                                Err(_) => return match opt {
                                    MmuOpt::Fetch => Err(Exception::FetchAccess(va as u64)),
                                    MmuOpt::Load => Err(Exception::LoadAccess(va as u64)),
                                    MmuOpt::Store => Err(Exception::StoreAccess(va as u64))
                                }
                            }
                        }
                        8 => {
                            match U64Access::write(&self.p.bus, pte_addr, leaf_pte.value() as u64) {
                                Ok(_) => {}
                                Err(_) => return match opt {
                                    MmuOpt::Fetch => Err(Exception::FetchAccess(va as u64)),
                                    MmuOpt::Load => Err(Exception::LoadAccess(va as u64)),
                                    MmuOpt::Store => Err(Exception::StoreAccess(va as u64))
                                }
                            }
                        }
                        _ => unreachable!()
                    }
                } else {
                    return match opt {
                        MmuOpt::Fetch => Err(Exception::FetchPageFault(va as u64)),
                        MmuOpt::Load => Err(Exception::LoadPageFault(va as u64)),
                        MmuOpt::Store => Err(Exception::StorePageFault(va as u64))
                    };
                }
            }
            //step 8
            let pa = Paddr::new(&vaddr, &leaf_pte, &info, level).value() as u64;
            if !self.check_pmp(pa, len as usize, opt, privilege) {
                match opt {
                    MmuOpt::Fetch => Err(Exception::FetchAccess(va as u64)),
                    MmuOpt::Load => Err(Exception::LoadAccess(va as u64)),
                    MmuOpt::Store => Err(Exception::StoreAccess(va as u64))
                }
            } else {
                Ok(pa)
            }
        } else {
            Ok(va as u64)
        }
    }
}





#[test]
fn pmp_basic_test() {
    let space = Arc::new(Space::new());
    let p = Processor::new(ProcessorCfg { xlen: XLen::X32, enabel_dirty: true }, &space);
    //no valid region
    assert_eq!(p.mmu().match_pmpcfg_entry(0, 1), None);
    //NA4
    p.state.csr_mut().pmpcfg0.set_bit_range(4, 3, PmpAType::NA4.into());
    p.state.csr_mut().pmpaddr0.set(0x8000_0000 >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x8000_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x8000_0000, 5).is_none());

    //NAPOT
    p.state.csr_mut().pmpcfg3.set_bit_range(4, 3, PmpAType::NAPOT.into());
    p.state.csr_mut().pmpaddr12.set((0x2000_0000 + 0x1_0000 - 1) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x2000_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2000_ffff, 1).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2000_ffff, 2).is_none());
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2000_ffff, 1), p.mmu().match_pmpcfg_entry(0x2000_0000, 4));
    assert_eq!(p.mmu().match_pmpcfg_entry(0x1000_ffff, 1), None);
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2001_0000, 4), None);
    //TOR
    p.state.csr_mut().pmpcfg3.set_bit_range(12, 11, PmpAType::TOR.into());
    p.state.csr_mut().pmpaddr13.set((0x2000_0000 + 0x1_0000) >> 2);
    p.state.csr_mut().pmpcfg3.set_bit_range(20, 19, PmpAType::TOR.into());
    p.state.csr_mut().pmpaddr14.set((0x2000_0000 + 0x2_0000) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x2001_0000, 4).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2001_ffff, 1).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2001_ffff, 2).is_none());
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2002_0000, 4), None);
    p.state.csr_mut().pmpcfg3.set_bit_range(23, 23, 1);
    assert!(p.mmu().match_pmpcfg_entry(0x2001_0000, 4).is_some());
}