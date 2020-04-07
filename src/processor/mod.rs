use std::collections::HashMap;
use terminus_macros::*;
use terminus_global::*;
use std::sync::Arc;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::any::TypeId;
use terminus_spaceport::EXIT_CTRL;

mod decode;

pub use decode::{Decoder, InsnMap, GDECODER, GlobalInsnMap, REGISTERY_INSN};

mod insn;

pub use insn::*;

pub mod trap;

use trap::{Exception, Trap, Interrupt};

mod extensions;

use extensions::*;
use extensions::i::csrs::*;

mod mmu;

use mmu::*;

mod fetcher;

use fetcher::*;

mod load_store;

use load_store::*;
use crate::system::{Bus, SimCmdSink, SimCtrlError, SimResp, SimCmd};

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

#[derive(Debug, Clone)]
pub struct ProcessorCfg {
    pub xlen: XLen,
    pub hartid: RegT,
    pub start_address: u64,
    pub privilege_level: PrivilegeLevel,
    pub enable_dirty: bool,
    pub extensions: Box<[char]>,
}

pub struct ProcessorStateSnapShot {
    pub config: ProcessorCfg,
    pub privilege: Privilege,
    pub xreg: [RegT; 32],
    pub pc: RegT,
    pub next_pc: RegT,
    pub ir: InsnT,
}

impl ProcessorStateSnapShot {
    pub fn trace(&self) -> String {
        format!("privilege = {:?};pc = {:#x}; ir = {:#x}; next_pc = {:#x};", self.privilege, self.pc, self.ir, self.next_pc)
    }
}

impl Display for ProcessorStateSnapShot {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "config:")?;
        writeln!(f, "{:#x?}", self.config)?;
        writeln!(f, "")?;
        writeln!(f, "states:")?;
        writeln!(f, "{}", self.trace())?;
        writeln!(f, "")?;
        writeln!(f, "registers:")?;
        for (i, v) in self.xreg.iter().enumerate() {
            writeln!(f, "   x{:<2} : {:#x}", i, v)?;
        }
        writeln!(f, "")?;
        Ok(())
    }
}

impl From<&ProcessorState> for ProcessorStateSnapShot {
    fn from(v: &ProcessorState) -> ProcessorStateSnapShot {
        ProcessorStateSnapShot {
            config: v.config.clone(),
            privilege: v.privilege(),
            xreg: v.xreg.borrow().clone(),
            pc: v.pc(),
            next_pc: v.next_pc(),
            ir: v.ir(),
        }
    }
}

pub struct ProcessorState {
    config: ProcessorCfg,
    privilege: RefCell<Privilege>,
    xreg: RefCell<[RegT; 32]>,
    extensions: HashMap<char, Extension>,
    pc: RefCell<RegT>,
    next_pc: RefCell<RegT>,
    ir: RefCell<InsnT>,
}

impl ProcessorState {
    fn new(config: ProcessorCfg) -> Result<ProcessorState, String> {
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
        for &ext in config.extensions.iter() {
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
        };

        state.csrs::<ICsrs>().unwrap().mhartid_mut().set(hartid);
        match state.config.privilege_level {
            PrivilegeLevel::MSU => {}
            PrivilegeLevel::MU => {
                state.csrs::<ICsrs>().unwrap().mstatus_mut().set_mpp_transform(|mpp| {
                    if mpp != 0 {
                        let m: u8 = Privilege::M.into();
                        m as RegT
                    } else {
                        0
                    }
                });
                state.csrs::<ICsrs>().unwrap().mstatus_mut().set_spp_transform(|_| { 0 });
                state.csrs::<ICsrs>().unwrap().mstatus_mut().set_tvm_transform(|_| { 0 });
            }
            PrivilegeLevel::M => {
                let m: u8 = Privilege::M.into();
                state.csrs::<ICsrs>().unwrap().mstatus_mut().set_mpp(m as RegT);
                state.csrs::<ICsrs>().unwrap().mstatus_mut().set_mpp_transform(move |_| {
                    m as RegT
                });
                state.csrs::<ICsrs>().unwrap().mstatus_mut().set_spp_transform(|_| { 0 });
                state.csrs::<ICsrs>().unwrap().mstatus_mut().set_tvm_transform(|_| { 0 });
            }
        }
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

    pub fn csr(&self, id: RegT) -> Result<RegT, Exception> {
        let trip_id = id & 0xfff;
        self.csr_privilege_check(trip_id)?;
        match self.extensions.values().find_map(|e| { e.csr_read(trip_id) }) {
            Some(v) => Ok(v),
            None => Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }

    pub fn set_csr(&self, id: RegT, value: RegT) -> Result<(), Exception> {
        let trip_id = id & 0xfff;
        self.csr_privilege_check(trip_id)?;
        match self.extensions.values().find_map(|e| { e.csr_write(trip_id, value) }) {
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

    pub fn check_privilege_level(&self, privilege: Privilege) -> Result<(), Exception> {
        match self.config().privilege_level {
            PrivilegeLevel::M => if privilege != Privilege::M {
                return Err(Exception::IllegalInsn(*self.ir.borrow()));
            },
            PrivilegeLevel::MU => if privilege == Privilege::S {
                return Err(Exception::IllegalInsn(*self.ir.borrow()));
            }
            PrivilegeLevel::MSU => {}
        }
        Ok(())
    }

    pub fn privilege(&self) -> Privilege {
        self.privilege.borrow().clone()
    }

    pub fn set_privilege(&self, privilege: Privilege) -> Privilege {
        match self.config().privilege_level {
            PrivilegeLevel::M => Privilege::M,
            PrivilegeLevel::MU => if privilege != Privilege::M {
                *self.privilege.borrow_mut() = Privilege::U;
                Privilege::U
            } else {
                *self.privilege.borrow_mut() = Privilege::M;
                Privilege::M
            }
            PrivilegeLevel::MSU => {
                *self.privilege.borrow_mut() = privilege;
                privilege
            }
        }
    }


    pub fn pc(&self) -> RegT {
        *self.pc.borrow()
    }

    pub fn set_pc(&self, pc: RegT) {
        *self.next_pc.borrow_mut() = pc
    }

    fn next_pc(&self) -> RegT {
        *self.next_pc.borrow()
    }

    fn ir(&self) -> InsnT {
        *self.ir.borrow()
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

pub struct Processor {
    state: Rc<ProcessorState>,
    cmd: SimCmdSink,
    mmu: Mmu,
    fetcher: Fetcher,
    load_store: LoadStore,
}

impl Processor {
    pub fn new(config: ProcessorCfg, bus: &Arc<Bus>, cmd: SimCmdSink) -> Processor {
        let state = match ProcessorState::new(config) {
            Ok(state) => Rc::new(state),
            Err(msg) => panic!(msg)
        };

        let mmu = Mmu::new(&state, bus);
        let fetcher = Fetcher::new(&state, bus);
        let load_store = LoadStore::new(&state, bus);
        Processor {
            state,
            cmd,
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

    fn one_insn(&self) -> Result<(), Exception> {
        let inst = self.fetcher.fetch(*self.state.next_pc.borrow(), self.mmu())?;
        *self.state.pc.borrow_mut() = *self.state.next_pc.borrow();
        *self.state.ir.borrow_mut() = inst.ir();
        inst.execute(self)
    }

    fn take_interrupt(&self) -> Result<(), Trap> {
        Ok(())
    }


    fn execute_one(&self) -> Result<(), Trap> {
        self.take_interrupt()?;
        match self.one_insn() {
            Ok(_) => Ok(()),
            Err(e) => Err(Trap::Exception(e))
        }
    }

    fn handle_trap(&self, trap: Trap) {
        let csrs = self.state().csrs::<ICsrs>().unwrap();
        let (int_flag, deleg, code, tval) = match trap {
            Trap::Exception(e) => (0 as RegT, csrs.medeleg().get(), e.code(), e.tval()),
            Trap::Interrupt(i) => (1 as RegT, csrs.mideleg().get(), i.code(), i.tval()),
        };
        //deleg to s-mode
        if self.state().privilege() != Privilege::M && (deleg >> code) & 1 == 1 {
            let tvec = csrs.stvec();
            let offset = if tvec.mode() == 1 && int_flag == 1 {
                code << 2
            } else {
                0
            };
            self.state().set_pc(tvec.base() << 2 + offset);
            csrs.scause_mut().set_code(code);
            csrs.scause_mut().set_int(int_flag);
            csrs.sepc_mut().set(self.state().pc());
            csrs.stval_mut().set(tval);

            let sie = csrs.mstatus().sie();
            csrs.mstatus_mut().set_spie(sie);
            let priv_value: u8 = self.state().privilege().into();
            csrs.mstatus_mut().set_spp(priv_value as RegT);
            csrs.mstatus_mut().set_sie(0);
            self.state().set_privilege(Privilege::S);
        } else {
            let tvec = csrs.mtvec();
            let offset = if tvec.mode() == 1 && int_flag == 1 {
                code << 2
            } else {
                0
            };
            self.state().set_pc(tvec.base() << 2 + offset);
            csrs.mcause_mut().set_code(code);
            csrs.mcause_mut().set_int(int_flag);
            csrs.mepc_mut().set(self.state().pc());
            csrs.mtval_mut().set(tval);

            let mie = csrs.mstatus().mie();
            csrs.mstatus_mut().set_mpie(mie);
            let priv_value: u8 = self.state().privilege().into();
            csrs.mstatus_mut().set_mpp(priv_value as RegT);
            csrs.mstatus_mut().set_mie(0);
            self.state().set_privilege(Privilege::M);
        }
    }

    fn step_one(&self) {
        if let Err(trap) = self.execute_one() {
            self.handle_trap(trap);
        }
    }

    fn handle_sim_cmd(&self) -> Result<SimResp, SimCtrlError> {
        let cmd = self.cmd.cmd().recv()?;
        if let Ok(msg) = EXIT_CTRL.poll() {
            return Ok(SimResp::Exited(msg, self.state().into()));
        }
        match cmd {
            SimCmd::RunOne => {
                self.step_one();
                Ok(SimResp::Resp(self.state().into()))
            }
            SimCmd::RunN(n) => {
                for _ in 0..n {
                    self.step_one();
                }
                Ok(SimResp::Resp(self.state().into()))
            }
            SimCmd::RunAll => {
                loop {
                    if let Ok(msg) = EXIT_CTRL.poll() {
                        return Ok(SimResp::Exited(msg, self.state().into()));
                    }
                    self.step_one();
                }
            }
        }
    }

    pub fn run(&self) -> Result<(), SimCtrlError> {
        loop {
            match self.handle_sim_cmd() {
                Ok(resp) => match resp {
                    SimResp::Exited(_, _) => {
                        self.cmd.resp().send(resp)?;
                        println!("haitid {} exited!", self.state().config().hartid);
                        return Ok(());
                    }
                    _ => {
                        self.cmd.resp().send(resp)?;
                    }
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
