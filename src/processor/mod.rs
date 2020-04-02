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
use std::fmt::{Display, Formatter};

#[cfg(test)]
mod test;

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Privilege {
    U = 0,
    S = 1,
    M = 3,
}

#[derive(Debug)]
pub struct ProcessorCfg {
    pub xlen: XLen,
    pub hartid: RegT,
    pub start_address: u64,
    pub enabel_dirty: bool,
}

pub struct ProcessorState {
    config: ProcessorCfg,
    privilege: RefCell<Privilege>,
    xreg: RefCell<[RegT; 32]>,
    extentions: RefCell<HashMap<char, Extension>>,
    basic_csr: RefCell<BasicCsr>,
    pc: RefCell<RegT>,
    ir: RefCell<InsnT>,
    pub bus: ProcessorBus,
}

impl ProcessorState {
    pub fn csrs(&self) -> Ref<'_, BasicCsr> {
        self.basic_csr.borrow()
    }
    pub fn csrs_mut(&self) -> RefMut<'_, BasicCsr> {
        self.basic_csr.borrow_mut()
    }

    fn csr_privilege_check(&self, id: RegT) -> Result<(), Exception> {
        let cur_priv: u8 = (*self.privilege.borrow()).into();
        let csr_priv: u8 = id.bit_range(9, 8);
        if cur_priv < csr_priv {
            return Err(Exception::IllegalInsn(*self.ir.borrow()));
        }
        Ok(())
    }
    pub fn csr(&self, id: RegT) -> Result<RegT, Exception> {
        let trip_id = id & 0xfff;
        self.csr_privilege_check(trip_id)?;
        match self.csrs().read(trip_id) {
            Some(v) => Ok(v),
            None => Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }
    pub fn set_csr(&self, id: RegT, value: RegT) -> Result<(), Exception> {
        let trip_id = id & 0xfff;
        self.csr_privilege_check(trip_id)?;
        match self.csrs_mut().write(trip_id, value) {
            Some(_) => Ok(()),
            None => Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }

    pub fn check_extension(&self, ext: char) -> bool {
        self.extentions.borrow().contains_key(&ext)
    }
    pub fn pc(&self) -> RegT {
        *self.pc.borrow()
    }
    pub fn set_pc(&self, pc: RegT) {
        *self.pc.borrow_mut() = pc
    }
    pub fn xreg(&self, id: RegT) -> RegT {
        let trip_id = id & 0x1f;
        if trip_id == 0 {
            0
        } else {
            (*self.xreg.borrow())[trip_id as usize]
        }
    }
    pub fn set_xreg(&self, id: RegT, value: RegT) {
        let trip_id = id & 0x1f;
        if trip_id != 0 {
            (*self.xreg.borrow_mut())[trip_id as usize] = value
        }
    }
}

impl Display for ProcessorState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "config:")?;
        writeln!(f, "{:#x?}", self.config)?;
        writeln!(f, "")?;
        writeln!(f, "privilege = {:?};pc = {:#x}; ir = {:#x}", *self.privilege.borrow(), *self.pc.borrow(), *self.ir.borrow())?;
        writeln!(f, "")?;
        writeln!(f, "registers:")?;
        for (i, v) in self.xreg.borrow().iter().enumerate() {
            writeln!(f, "   x{:<2} : {:#x}", i, v)?;
        }
        writeln!(f, "")?;
        Ok(())
    }
}

pub struct Processor {
    state: Rc<ProcessorState>,
    mmu: Mmu,
    fetcher: Fetcher,
}

impl Processor {
    pub fn new(config: ProcessorCfg, space: &Arc<Space>) -> Processor {
        let ProcessorCfg { xlen, hartid, start_address, enabel_dirty } = config;
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
            ir: RefCell::new(0),
            bus: ProcessorBus::new(space),
        });
        state.csrs_mut().mhartid.set(hartid);
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

    pub fn state(&self) -> &ProcessorState {
        self.state.deref()
    }

    pub fn execute_one(&self) -> Result<(), Exception> {
        let inst = self.fetcher.fetch(*self.state.pc.borrow(), self.mmu())?;
        *self.state.ir.borrow_mut() = inst.ir();
        inst.execute(self)
    }

    fn handle_exception(&self, expt: Exception) {}

    pub fn step_one(&self) {
        match self.execute_one() {
            Ok(_) => {}
            Err(e) => self.handle_exception(e)
        }
    }
}
