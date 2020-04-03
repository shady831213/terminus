use std::collections::HashMap;
use terminus_macros::*;
use terminus_global::*;
use terminus_spaceport::space::Space;
use std::sync::Arc;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::any::TypeId;


mod decode;

pub use decode::{Decoder, InsnMap, GDECODER, GlobalInsnMap, REGISTERY_INSN};

mod insn;

pub use insn::*;

pub mod execption;

use execption::Exception;

mod extensions;

use extensions::*;
use extensions::i::csrs::*;


mod mmu;

use mmu::*;

mod bus;

use bus::*;

mod fetcher;

use fetcher::*;

mod load_store;

use load_store::*;

#[cfg(test)]
mod test;

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    M = 1,
    MU = 2,
    MSU = 3,
}

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
    pub privilege_level: PrivilegeLevel,
    pub enabel_dirty: bool,
}

pub struct ProcessorState {
    config: ProcessorCfg,
    privilege: RefCell<Privilege>,
    xreg: RefCell<[RegT; 32]>,
    extensions: HashMap<char, Extension>,
    pc: RefCell<RegT>,
    next_pc: RefCell<RegT>,
    ir: RefCell<InsnT>,
    pub bus: ProcessorBus,
}

impl ProcessorState {
    fn new(config: ProcessorCfg, space: &Arc<Space>, extensions: Vec<char>) -> Result<ProcessorState, String> {
        let hartid = config.hartid;
        let start_address = config.start_address;
        if config.xlen == XLen::X32 && start_address.leading_zeros() < 32 {
            return Err(format!("invalid start addr {:#x} when xlen == X32!", start_address));
        }
        let mut extensions_map: HashMap<char, Extension> = HashMap::new();
        let mut add_extension = |id: char| -> Result<(), String>  {
            let ext = Extension::new(&config, id)?;
            extensions_map.insert(id, ext);
            Ok(())
        };
        add_extension('i')?;
        for ext in extensions {
            add_extension(ext)?
        }
        let state = ProcessorState {
            config,
            privilege: RefCell::new(Privilege::M),
            xreg: RefCell::new([0 as RegT; 32]),
            extensions: extensions_map,
            pc: RefCell::new(0),
            next_pc: RefCell::new(start_address),
            ir: RefCell::new(0),
            bus: ProcessorBus::new(space),
        };

        state.csrs::<ICsrs>().unwrap().mhartid_mut().set(hartid);
        Ok(state)
    }


    fn csrs<T: 'static>(&self) -> Result<Rc<T>, String> {
        if let Some(t) = self.extensions.values().find_map(|extension| {
            if let Some(csrs) = extension.csrs() {
                match csrs.downcast::<T>() {
                    Ok(t) => Some(t),
                    Err(_) => None
                }
            } else {
                None
            }
        }) {
            Ok(t)
        } else {
            Err(format!("can not find csrs {:?}", TypeId::of::<T>()))
        }
    }

    pub fn config(&self) -> &ProcessorCfg {
        &self.config
    }

    fn csr_privilege_check(&self, id: RegT) -> Result<(), Exception> {
        let cur_priv: u8 = (*self.privilege.borrow()).into();
        let csr_priv: u8 = id.bit_range(9, 8);
        if cur_priv < csr_priv {
            return Err(Exception::IllegalInsn(*self.ir.borrow()));
        }
        Ok(())
    }

    fn csr_handle_config(&self, id: RegT, value: RegT) -> RegT {
        if id == 0x300 {
            let mut mstatus = value;
            match self.config().privilege_level {
                PrivilegeLevel::MSU => {}
                PrivilegeLevel::MU => {
                    //mpp field
                    mstatus.set_bit_range(12, 11, 1);
                    //spp field
                    mstatus.set_bit_range(8, 8, 0);
                    //tvm field
                    mstatus.set_bit_range(20, 20, 0);
                }
                PrivilegeLevel::M => {
                    //mpp field
                    mstatus.set_bit_range(12, 11, 0);
                    //spp field
                    mstatus.set_bit_range(8, 8, 0);
                    //tvm field
                    mstatus.set_bit_range(20, 20, 0);
                }
            };
            mstatus
        } else {
            value
        }
    }

    pub fn csr(&self, id: RegT) -> Result<RegT, Exception> {
        let trip_id = id & 0xfff;
        self.csr_privilege_check(trip_id)?;
        match self.extensions.values().find_map(|e| { e.csr_read(trip_id) }) {
            Some(v) => Ok(self.csr_handle_config(trip_id, v)),
            None => Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }

    pub fn set_csr(&self, id: RegT, value: RegT) -> Result<(), Exception> {
        let trip_id = id & 0xfff;
        self.csr_privilege_check(trip_id)?;
        match self.extensions.values().find_map(|e| { e.csr_write(trip_id, self.csr_handle_config(trip_id, value)) }) {
            Some(_) => Ok(()),
            None => Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }

    pub fn check_extension(&self, ext: char) -> Result<(), Exception> {
        if self.extensions.contains_key(&ext) {
            Ok(())
        } else {
            Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }

    pub fn check_xlen(&self, xlen: XLen) -> Result<(), Exception> {
        if xlen == self.config().xlen {
            Ok(())
        } else {
            Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }

    pub fn check_privilege_level(&self, privilege_level: PrivilegeLevel) -> Result<(), Exception> {
        let config:u8 = self.config().privilege_level.into();
        let check:u8 = privilege_level.into();
        if config < check {
            Err(Exception::IllegalInsn(*self.ir.borrow()))
        } else {
            Ok(())
        }
    }

    pub fn pc(&self) -> RegT {
        *self.pc.borrow()
    }

    pub fn set_pc(&self, pc: RegT) {
        *self.next_pc.borrow_mut() = pc
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

    //fixme
    pub fn trace(&self) -> String {
        format!("privilege = {:?};pc = {:#x}; ir = {:#x}; next_pc = {:#x};", *self.privilege.borrow(), *self.pc.borrow(), *self.ir.borrow(), *self.next_pc.borrow())
    }
}

impl Display for ProcessorState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "config:")?;
        writeln!(f, "{:#x?}", self.config)?;
        writeln!(f, "")?;
        writeln!(f, "extensions:")?;
        writeln!(f, "{:?}", self.extensions.keys())?;
        writeln!(f, "")?;
        writeln!(f, "states:")?;
        writeln!(f, "{}", self.trace())?;
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
    load_store: LoadStore,
}

impl Processor {
    pub fn new(config: ProcessorCfg, space: &Arc<Space>, extensions: Vec<char>) -> Processor {
        let state = match ProcessorState::new(config, space, extensions) {
            Ok(state) => Rc::new(state),
            Err(msg) => panic!(msg)
        };

        let mmu = Mmu::new(&state);
        let fetcher = Fetcher::new(&state);
        let load_store = LoadStore::new(&state);

        Processor {
            state,
            mmu,
            fetcher,
            load_store,
        }
    }

    pub fn mmu(&self) -> &Mmu {
        &self.mmu
    }

    pub fn load_store(&self) -> &LoadStore {
        &self.load_store
    }


    pub fn state(&self) -> &ProcessorState {
        self.state.deref()
    }

    pub fn execute_one(&self) -> Result<(), Exception> {
        let inst = self.fetcher.fetch(*self.state.next_pc.borrow(), self.mmu())?;
        *self.state.pc.borrow_mut() = *self.state.next_pc.borrow();
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
