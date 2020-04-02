use terminus_global::{XLen, RegT};
use crate::processor::mmu::Mmu;
use std::marker::PhantomData;
use std::ops::Deref;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use terminus_macros::*;
use crate::processor::extensions::i::csrs::*;

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

pub struct PmpCfgsIter<'m> {
    mmu: &'m Mmu,
    idx: u8,
    marker: PhantomData<&'m Mmu>,
}


impl<'m> PmpCfgsIter<'m> {
    pub fn new(mmu: &'m Mmu, marker: PhantomData<&'m Mmu>) -> PmpCfgsIter<'m> {
        PmpCfgsIter {
            mmu,
            idx: 0,
            marker,
        }
    }
    fn get_cfg(&self, csr: &ICsrs) -> RegT {
        match csr.xlen {
            XLen::X32 => {
                match (self.idx >> 2) & 0x3 {
                    0 => csr.pmpcfg0().get(),
                    1 => csr.pmpcfg1().get(),
                    2 => csr.pmpcfg2().get(),
                    3 => csr.pmpcfg3().get(),
                    _ => unreachable!()
                }
            }
            XLen::X64 => {
                match (self.idx >> 3) & 0x1 {
                    0 => csr.pmpcfg0().get(),
                    1 => csr.pmpcfg2().get(),
                    _ => unreachable!()
                }
            }
        }
    }

    fn get_entry(&self) -> PmpCfgEntry {
        let csr = self.mmu.p.csrs::<ICsrs>().unwrap();
        let offset: u8 = match csr.xlen {
            XLen::X32 => self.idx.bit_range(1, 0),
            XLen::X64 => self.idx.bit_range(2, 0),
        };
        let cfg:u8 = self.get_cfg(csr.deref()).bit_range(((offset as usize) << 3) + 7, (offset as usize) << 3);
        cfg.into()
    }
}

impl<'m> Iterator for PmpCfgsIter<'m> {
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