use super::*;
use crate::Exception;
use std::marker::PhantomData;
use terminus_global::*;
use terminus_macros::*;
use std::convert::TryFrom;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub struct Mmu<'p> {
    p: &'p Processor,
    marker: PhantomData<&'p Processor>,
}

impl<'p> Mmu<'p> {
    pub fn new(p: &'p Processor, marker: PhantomData<&'p Processor>) -> Mmu<'p> {
        Mmu {
            p,
            marker,
        }
    }

    fn pmpcfgs_iter<'m>(&'m self) -> PmpCfgsIter<'m, 'p> {
        PmpCfgsIter {
            mmu: &self,
            idx: 0,
            marker: PhantomData,
        }
    }

    fn get_pmpaddr(&self, idx: u8) -> RegT {
        self.p.basic_csr.read(0x3b0 + idx as RegT).unwrap()
    }

    fn match_pmpcfg_entry<F: Fn(&Processor, &PmpCfgEntry) -> bool>(&self, addr: u64, condition: F) -> Option<PmpCfgEntry> {
        let trail_addr = addr >> 2;
        self.pmpcfgs_iter().enumerate()
            .find(|(idx, entry)| {
                condition(self.p, entry) && match PmpAType::try_from(entry.a()).unwrap() {
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
            .map(|(_, entry)| { entry })
    }
}


#[derive(IntoPrimitive, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum PmpAType {
    OFF = 0,
    TOR = 1,
    NA4 = 2,
    NAPOT = 3,
}

bitfield! {
#[derive(Eq,PartialEq)]
pub struct PmpCfgEntry(u8);
impl Debug;
pub r, set_r:0, 0;
pub w, set_w:1, 1;
pub x, set_x:2, 2;
pub a, set_a:4,3;
pub l, set_l:7, 7;
}

impl From<u8> for PmpCfgEntry {
    fn from(v: u8) -> Self {
        PmpCfgEntry(v)
    }
}

pub struct PmpCfgsIter<'m, 'p> {
    mmu: &'m Mmu<'p>,
    idx: u8,
    marker: PhantomData<&'m Mmu<'p>>,
}


impl<'a, 'b> PmpCfgsIter<'a, 'b> {
    fn get_cfg(&self) -> &PmpCfg {
        match self.mmu.p.xlen {
            XLen::X32 => {
                match (self.idx >> 2) & 0x3 {
                    0 => &self.mmu.p.basic_csr.pmpcfg0,
                    1 => &self.mmu.p.basic_csr.pmpcfg1,
                    2 => &self.mmu.p.basic_csr.pmpcfg2,
                    3 => &self.mmu.p.basic_csr.pmpcfg3,
                    _ => unreachable!()
                }
            }
            XLen::X64 => {
                match (self.idx >> 3) & 0x1 {
                    0 => &self.mmu.p.basic_csr.pmpcfg0,
                    1 => &self.mmu.p.basic_csr.pmpcfg2,
                    _ => unreachable!()
                }
            }
        }
    }

    fn get_entry(&self) -> PmpCfgEntry {
        let offset: u8 = match self.mmu.p.xlen {
            XLen::X32 => self.idx.bit_range(1, 0),
            XLen::X64 => self.idx.bit_range(2, 0),
        };
        (self.get_cfg().bit_range(((offset as usize) << 3) + 7, (offset as usize) << 3)).into()
    }
}

impl<'a, 'b> Iterator for PmpCfgsIter<'a, 'b> {
    type Item = PmpCfgEntry;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 15 {
            None
        } else {
            let res = self.get_entry();
            self.idx += 1;
            Some(res)
        }
    }
}

#[test]
fn pmp_basic_test() {
    let mut p = Processor::new(XLen::X32);
    //no valid region
    assert_eq!(p.mmu().match_pmpcfg_entry(0, |p, entry| { true }), None);
    //NA4
    p.basic_csr.pmpcfg0.set_bit_range(4, 3, PmpAType::NA4.into());
    p.basic_csr.pmpaddr0.set(0x8000_0000 >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x8000_0000, |p, entry| { true }).is_some());
    //NAPOT
    p.basic_csr.pmpcfg3.set_bit_range(4, 3, PmpAType::NAPOT.into());
    p.basic_csr.pmpaddr12.set((0x2000_0000 + 0x1_0000 - 1) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x2000_0000, |p, entry| { true }).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2000_ffff, |p, entry| { true }).is_some());
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2000_ffff, |p, entry| { true }), p.mmu().match_pmpcfg_entry(0x2000_0000, |p, entry| { true }));
    assert_eq!(p.mmu().match_pmpcfg_entry(0x1000_ffff, |p, entry| { true }), None);
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2001_0000, |p, entry| { true }), None);
    //TOR
    p.basic_csr.pmpcfg3.set_bit_range(12, 11, PmpAType::TOR.into());
    p.basic_csr.pmpaddr13.set((0x2000_0000 + 0x1_0000) >> 2);
    p.basic_csr.pmpcfg3.set_bit_range(20, 19, PmpAType::TOR.into());
    p.basic_csr.pmpaddr14.set((0x2000_0000 + 0x2_0000) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x2001_0000, |p, entry| { true }).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2001_ffff, |p, entry| { true }).is_some());
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2002_0000, |p, entry| { true }), None);
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2001_0000, |p, entry| { entry.l() == 1 }), None);
    p.basic_csr.pmpcfg3.set_bit_range(23, 23, 1);
    assert!(p.mmu().match_pmpcfg_entry(0x2001_0000, |p, entry| { entry.l() == 1 }).is_some());

}