use std::collections::HashMap;
use super::extentions::Extension;
use terminus_macros::*;
use terminus_global::*;
use std::marker::PhantomData;
use terminus_spaceport::space::Space;
use std::sync::Arc;

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
    pub bus: ProcessorBus,
}

impl Processor {
    pub fn new(xlen: XLen, space: &Arc<Space>) -> Processor {
        Processor {
            xreg: [0 as RegT; 32],
            extentions: HashMap::new(),
            basic_csr: BasicCsr::new(xlen),
            xlen,
            bus: ProcessorBus::new(space),
        }
    }

    pub fn mmu(&self) -> Mmu {
        Mmu::new(self, PhantomData)
    }
}
