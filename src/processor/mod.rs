use std::collections::HashMap;
use super::extentions::Extension;
use terminus_macros::*;
use terminus_global::*;
use terminus_spaceport::space::Space;
use std::sync::Arc;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};

mod csr;

use csr::*;

mod mmu;

use mmu::*;

mod bus;

use bus::*;

mod fetcher;

use fetcher::*;
use crate::Exception;

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
    pub start_address: u64,
    pub enabel_dirty: bool,
}

pub struct ProcessorState {
    config: ProcessorCfg,
    privilege: RefCell<Privilege>,
    pub xreg: RefCell<[RegT; 32]>,
    extentions: RefCell<HashMap<char, Extension>>,
    basic_csr: RefCell<BasicCsr>,
    pc: RefCell<RegT>,
    pub bus: ProcessorBus,
}

impl ProcessorState {
    pub fn csr(&self) -> Ref<'_, BasicCsr> {
        self.basic_csr.borrow()
    }
    pub fn csr_mut(&self) -> RefMut<'_, BasicCsr> {
        self.basic_csr.borrow_mut()
    }
    pub fn check_extension(&self, ext: char) -> bool {
        self.extentions.borrow().contains_key(&ext)
    }
}

pub struct Processor {
    state: Rc<ProcessorState>,
    mmu: Mmu,
    fetcher: Fetcher,
}

impl Processor {
    pub fn new(config: ProcessorCfg, space: &Arc<Space>) -> Processor {
        let xlen = config.xlen;
        let start_address = config.start_address;
        if xlen == XLen::X32 && start_address.leading_zeros() < 32 {
            panic!(format!("invalid start addr {:#x} when xlen == X32!", start_address))
        }
        let state = Rc::new(ProcessorState {
            config,
            privilege: RefCell::new(Privilege::M),
            xreg: RefCell::new([0 as RegT; 32]),
            extentions: RefCell::new(HashMap::new()),
            basic_csr: RefCell::new(BasicCsr::new(xlen)),
            pc: RefCell::new(start_address),
            bus: ProcessorBus::new(space),
        });
        let mmu = Mmu::new(&state);
        let fetcher = Fetcher::new(&state);
        Processor {
            state,
            mmu,
            fetcher,
        }
    }

    pub fn mmu(&self) -> &Mmu {
        &self.mmu
    }

    pub fn execute_one(&self) -> Result<RegT, Exception> {
        let inst = self.fetcher.fetch(*self.state.pc.borrow(), self.mmu())?;
        inst.execute(self)
    }

    fn handle_exception(&self, expt: Exception) {}

    pub fn step_one(&self) {
        match self.execute_one() {
            Ok(next_pc) => *self.state.pc.borrow_mut() = next_pc,
            Err(e) => self.handle_exception(e)
        }
    }
}
