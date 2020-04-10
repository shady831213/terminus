use std::collections::HashMap;
use terminus_macros::*;
use terminus_global::*;
use std::sync::Arc;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::fmt::{Display, Formatter};
use std::any::TypeId;
use terminus_spaceport::irq::IrqVec;

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
use crate::system::Bus;

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
    pub privilege_level: PrivilegeLevel,
    pub enable_dirty: bool,
    pub extensions: Box<[char]>,
}


pub struct ProcessorState {
    hartid: usize,
    start_address: u64,
    config: ProcessorCfg,
    privilege: RefCell<Privilege>,
    xreg: RefCell<[RegT; 32]>,
    extensions: RefCell<HashMap<char, Extension>>,
    pc: RefCell<RegT>,
    next_pc: RefCell<RegT>,
    ir: RefCell<InsnT>,
    clint: Arc<IrqVec>,
}

impl ProcessorState {
    pub fn trace(&self) -> String {
        format!("privilege = {:?};pc = {:#x}; ir = {:#x}; next_pc = {:#x};", self.privilege(), self.pc(), self.ir(), self.next_pc())
    }
}

impl Display for ProcessorState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "hartid:{}", self.hartid)?;
        writeln!(f, "start_address:{}", self.start_address)?;
        writeln!(f, "config:")?;
        writeln!(f, "{:#x?}", self.config)?;
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


impl ProcessorState {
    fn new(hartid: usize, start_address: u64, config: ProcessorCfg, clint: &Arc<IrqVec>) -> Result<ProcessorState, String> {
        if config.xlen == XLen::X32 && start_address.leading_zeros() < 32 {
            return Err(format!("invalid start addr {:#x} when xlen == X32!", start_address));
        }
        let state = ProcessorState {
            hartid,
            start_address,
            config,
            privilege: RefCell::new(Privilege::M),
            xreg: RefCell::new([0 as RegT; 32]),
            extensions: RefCell::new(HashMap::new()),
            pc: RefCell::new(0),
            next_pc: RefCell::new(start_address),
            ir: RefCell::new(0),
            clint: clint.clone(),
        };
        state.reset()?;
        Ok(state)
    }

    fn reset(&self) -> Result<(), String> {
        macro_rules! deleg_csr_set {
                    ($src:ident, $tar:ident, $setter:ident, $transform:ident) => {
                        self.csrs::<ICsrs>().unwrap().$src().$transform({
                        let _csrs = self.csrs::<ICsrs>().unwrap();
                            move |field| {
                                _csrs.$tar().$setter(field);
                                field
                            }
                        });
                    }
                };
        macro_rules! deleg_csr_get {
                    ($src:ident, $tar:ident, $getter:ident, $transform:ident) => {
                        self.csrs::<ICsrs>().unwrap().$src().$transform({
                        let _csrs = self.csrs::<ICsrs>().unwrap();
                            move |_| {
                                _csrs.$tar().$getter()
                            }
                        });
                    }
                };
        macro_rules! deleg_csr {
                    ($src:ident, $get_tar:ident, $getter:ident, $get_transform:ident, $set_tar:ident, $setter:ident, $set_transform:ident) => {
                        deleg_csr_get!($src,$get_tar, $getter, $get_transform);
                        deleg_csr_set!($src,$set_tar, $setter, $set_transform);
                    }
                };

        *self.xreg.borrow_mut() = [0 as RegT; 32];
        *self.extensions.borrow_mut() = HashMap::new();
        *self.pc.borrow_mut() = 0;
        *self.next_pc.borrow_mut() = self.start_address;
        *self.ir.borrow_mut() = 0;
        self.add_extension()?;
        //register clint:0:msip, 1:mtip
        self.csrs::<ICsrs>().unwrap().mip_mut().msip_transform({
            let clint = self.clint.clone();
            move |_| {
                clint.pending(0).unwrap() as RegT
            }
        });
        self.csrs::<ICsrs>().unwrap().mip_mut().mtip_transform({
            let clint = self.clint.clone();
            move |_| {
                clint.pending(1).unwrap() as RegT
            }
        });
        //hartid
        self.csrs::<ICsrs>().unwrap().mhartid_mut().set(self.hartid as RegT);

        //xlen config
        match self.config().xlen {
            XLen::X32 => {
                self.csrs::<ICsrs>().unwrap().misa_mut().set_mxl(1);
            }
            XLen::X64 => {
                self.csrs::<ICsrs>().unwrap().misa_mut().set_mxl(2);
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_uxl(2);
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_sxl(2);
                self.csrs::<ICsrs>().unwrap().sstatus_mut().set_uxl(2);
            }
        }
        //extensions config
        let extensions_value = self.extensions().keys()
            .map(|e| { (*e as u8 - 'a' as u8) as RegT })
            .map(|v| { 1 << v })
            .fold(0 as RegT, |acc, v| { acc | v });
        self.csrs::<ICsrs>().unwrap().misa_mut().set_extensions(extensions_value);
        //privilege_level config
        macro_rules! deleg_sstatus {
                    ($getter:ident, $get_transform:ident, $setter:ident, $set_transform:ident) => {
                        deleg_csr!(sstatus_mut,mstatus, $getter, $get_transform, mstatus_mut, $setter, $set_transform);
                    }
                };
        deleg_sstatus!(upie, upie_transform, set_upie, set_upie_transform);
        deleg_sstatus!(sie, sie_transform, set_sie, set_sie_transform);
        deleg_sstatus!(upie, upie_transform, set_upie, set_upie_transform);
        deleg_sstatus!(spie, spie_transform, set_spie, set_spie_transform);
        deleg_sstatus!(spp, spp_transform, set_spp, set_spp_transform);
        deleg_sstatus!(fs, fs_transform, set_fs, set_fs_transform);
        deleg_sstatus!(xs, xs_transform, set_xs, set_xs_transform);
        deleg_sstatus!(sum, sum_transform, set_sum, set_sum_transform);
        deleg_sstatus!(mxr, mxr_transform, set_mxr, set_mxr_transform);
        deleg_sstatus!(sd, sd_transform, set_sd, set_sd_transform);
        deleg_sstatus!(uxl, uxl_transform, set_uxl, set_uxl_transform);
        match self.config.privilege_level {
            PrivilegeLevel::MSU => {}
            PrivilegeLevel::MU => {
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_mpp_transform(|mpp| {
                    if mpp != 0 {
                        let m: u8 = Privilege::M.into();
                        m as RegT
                    } else {
                        0
                    }
                });
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_spp_transform(|_| { 0 });
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_tvm_transform(|_| { 0 });
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_tsr_transform(|_| { 0 });
            }
            PrivilegeLevel::M => {
                let m: u8 = Privilege::M.into();
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_mpp(m as RegT);
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_mpp_transform(move |_| {
                    m as RegT
                });
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_spp_transform(|_| { 0 });
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_tvm_transform(|_| { 0 });
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_tsr_transform(|_| { 0 });
                self.csrs::<ICsrs>().unwrap().mstatus_mut().set_tw_transform(|_| { 0 });
            }
        }
        Ok(())
    }

    fn add_extension(&self) -> Result<(), String> {
        let add_one_extension = |id: char| -> Result<(), String>  {
            let ext = Extension::new(self.config(), id)?;
            self.extensions.borrow_mut().insert(id, ext);
            Ok(())
        };
        add_one_extension('i')?;
        for &ext in self.config().extensions.iter() {
            add_one_extension(ext)?
        }
        Ok(())
    }

    fn extensions(&self) -> Ref<'_, HashMap<char, Extension>> {
        self.extensions.borrow()
    }

    fn csrs<T: 'static>(&self) -> Result<Rc<T>, String> {
        if let Some(t) = self.extensions().values().find_map(|extension| {
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
        //stap
        if id == 0x180 && self.privilege() == Privilege::S && self.csrs::<ICsrs>().unwrap().mstatus().tvm() == 1 {
            return Err(Exception::IllegalInsn(*self.ir.borrow()));
        }
        Ok(())
    }

    pub fn csr(&self, id: RegT) -> Result<RegT, Exception> {
        let trip_id = id & 0xfff;
        self.csr_privilege_check(trip_id)?;
        match self.extensions().values().find_map(|e| { e.csr_read(trip_id) }) {
            Some(v) => Ok(v),
            None => Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }

    pub fn set_csr(&self, id: RegT, value: RegT) -> Result<(), Exception> {
        let trip_id = id & 0xfff;
        self.csr_privilege_check(trip_id)?;
        match self.extensions().values().find_map(|e| { e.csr_write(trip_id, value) }) {
            Some(_) => Ok(()),
            None => Err(Exception::IllegalInsn(*self.ir.borrow()))
        }
    }

    pub fn check_extension(&self, ext: char) -> Result<(), Exception> {
        if self.extensions().contains_key(&ext) {
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
    mmu: Mmu,
    fetcher: Fetcher,
    load_store: LoadStore,
}

impl Processor {
    pub fn new(hartid: usize, start_address: u64, config: ProcessorCfg, bus: &Arc<Bus>, clint: &Arc<IrqVec>) -> Processor {
        let state = match ProcessorState::new(hartid, start_address, config, clint) {
            Ok(state) => Rc::new(state),
            Err(msg) => panic!(msg)
        };

        let mmu = Mmu::new(&state, bus);
        let fetcher = Fetcher::new(&state, bus);
        let load_store = LoadStore::new(&state, bus);
        Processor {
            state,
            mmu,
            fetcher,
            load_store,
        }
    }

    pub fn reset(&self) {
        if let Err(msg) = self.state.reset() {
            panic!(msg)
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
        *self.state.pc.borrow_mut() = *self.state.next_pc.borrow();
        let inst = self.fetcher.fetch(*self.state.pc.borrow(), self.mmu())?;
        *self.state.ir.borrow_mut() = inst.ir();
        inst.execute(self)
    }

    fn take_interrupt(&self) -> Result<(), Interrupt> {
        let csrs = self.state().csrs::<ICsrs>().unwrap();
        let pendings = csrs.mip().get() & csrs.mie().get();
        let mie = csrs.mstatus().mie();
        let m_enabled = self.state().privilege() != Privilege::M || (self.state().privilege() == Privilege::M && mie == 1);
        let m_pendings = pendings & !csrs.mideleg().get() & sext(m_enabled as RegT, 1);
        let sie = csrs.mstatus().sie();
        let s_enabled = self.state().privilege() == Privilege::U || (self.state().privilege() == Privilege::S && sie == 1);
        let s_pendings = pendings & csrs.mideleg().get() & sext(s_enabled as RegT, 1);

        //m_pendings > s_pendings
        let interrupts = Mip::new(self.state().config().xlen,
                                  if m_pendings == 0 {
                                      s_pendings
                                  } else {
                                      m_pendings
                                  });

        // MEI > MSI > MTI > SEI > SSI > STI
        if interrupts.meip() == 1 {
            return Err(Interrupt::MEInt);
        } else if interrupts.msip() == 1 {
            return Err(Interrupt::MSInt);
        } else if interrupts.mtip() == 1 {
            return Err(Interrupt::MTInt);
        } else if interrupts.seip() == 1 {
            return Err(Interrupt::SEInt);
        } else if interrupts.ssip() == 1 {
            return Err(Interrupt::SSInt);
        } else if interrupts.stip() == 1 {
            return Err(Interrupt::STInt);
        }
        Ok(())
    }


    fn execute_one(&self) -> Result<(), Trap> {
        self.take_interrupt()?;
        self.one_insn()?;
        Ok(())
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
            self.state().set_pc((tvec.base() << 2) + offset);
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
            self.state().set_pc((tvec.base() << 2) + offset);
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

    pub fn step(&self, n: usize) {
        assert!(n > 0);
        for _ in 0..n {
            self.step_one()
        }
    }
}
