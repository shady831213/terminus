use std::collections::HashMap;
use super::extentions::Extension;
use terminus_macros::*;
use terminus_global::*;
use terminus_spaceport::space::Space;
use std::sync::Arc;
use num_enum::{IntoPrimitive, TryFromPrimitive};

mod csr;

use csr::*;

mod mmu;

use mmu::*;

mod bus;

use bus::*;
use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};
use std::ops::DerefMut;


#[cfg(test)]
mod test;

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Privilege {
    U = 0,
    S = 1,
    M = 3,
}

pub struct ProcessorCfg {
    pub xlen: XLen,
    pub enabel_dirty: bool,
}

pub struct ProcessorState {
    config: ProcessorCfg,
    privilege: RefCell<Privilege>,
    pub xreg: RefCell<[RegT; 32]>,
    extentions: RefCell<HashMap<char, Extension>>,
    basic_csr: RefCell<BasicCsr>,
    pub bus: ProcessorBus,
}

impl ProcessorState {
    pub fn csr(&self) -> Ref<'_, BasicCsr> {
        self.basic_csr.borrow()
    }
    pub fn csr_mut(&self) -> RefMut<'_, BasicCsr> {
        self.basic_csr.borrow_mut()
    }
}

pub struct Processor {
    state: Rc<ProcessorState>,
    mmu: Mmu,
}

impl Processor {
    pub fn new(config: ProcessorCfg, space: &Arc<Space>) -> Processor {
        let xlen = config.xlen;
        let state = Rc::new(ProcessorState {
            config,
            privilege: RefCell::new(Privilege::M),
            xreg: RefCell::new([0 as RegT; 32]),
            extentions: RefCell::new(HashMap::new()),
            basic_csr: RefCell::new(BasicCsr::new(xlen)),
            bus: ProcessorBus::new(space),
        });
        let mmu = Mmu::new(&state);
        Processor {
            state,
            mmu,
        }
    }

    pub fn mmu(&self) -> &Mmu {
        &self.mmu
    }
}
