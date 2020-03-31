use std::collections::HashMap;
use super::extentions::Extension;
use terminus_macros::*;
use terminus_global::*;
use std::marker::PhantomData;

mod csr;
use csr::*;

mod mmu;
use mmu::*;

mod bus;
use bus::*;

#[cfg(test)]
mod test;

pub struct Processor {
    pub xreg: [RegT; 32],
    extentions: HashMap<char, Extension>,
    pub basic_csr: BasicCsr,
    pub xlen: XLen,
}

impl Processor {
    pub fn new(xlen: XLen) -> Processor {
        Processor {
            xreg: [0 as RegT; 32],
            extentions: HashMap::new(),
            basic_csr: BasicCsr::new(xlen),
            xlen,
        }
    }

    pub fn mmu(&self) -> Mmu {
        Mmu::new(self, PhantomData)
    }
}
