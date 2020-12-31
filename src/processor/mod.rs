use crate::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::io::Write;
use terminus_spaceport::irq::IrqVec;
use crate::devices::bus::Bus;
use std::mem::MaybeUninit;

pub mod privilege;
use privilege::*;

pub mod trap;

use trap::{Exception, Trap, Interrupt};

pub mod extensions;

use extensions::*;


mod mmu;

use mmu::*;

mod fetcher;

use fetcher::*;

mod load_store;

use load_store::*;

trait HasCsr {
    fn csr_write(&self, state: &ProcessorState, addr: InsnT, value: RegT) -> Option<()>;
    fn csr_read(&self, state: &ProcessorState, addr: InsnT) -> Option<RegT>;
}

trait NoCsr {
    fn csr_write(&self, _: &ProcessorState, _: InsnT, _: RegT) -> Option<()> {
        None
    }
    fn csr_read(&self, _: &ProcessorState, _: InsnT) -> Option<RegT> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorCfg {
    pub xlen: XLen,
    pub enable_dirty: bool,
    pub extensions: Box<[char]>,
    pub freq: usize,
}

pub struct ProcessorState {
    hartid: usize,
    config: ProcessorCfg,
    privilege: PrivilegeStates,
    xreg: [RegT; 32],
    extensions: [Extension; 26],
    pc: RegT,
    next_pc: RegT,
    ir: InsnT,
    insns_cnt: Rc<RefCell<u64>>,
    clint: Option<IrqVec>,
    plic: Option<IrqVec>,
    wfi: bool,
}

impl ProcessorState {
    pub fn trace(&self) -> String {
        format!("hartid = {}; privilege = {:?};pc = {:#x}; ir = {:#x}; next_pc = {:#x}; insns_cnt = {};", self.hartid, self.privilege(), self.pc(), *self.ir(), self.next_pc(), *self.insns_cnt().borrow())
    }
}

impl Display for ProcessorState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "hartid:{}", self.hartid)?;
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
        if let Extension::F(ref float) = self.get_extension('f') {
            for (i, v) in float.fregs().iter().enumerate() {
                writeln!(f, "   f{:<2} : {:#x}", i, v)?;
            }
        }
        writeln!(f, "")?;
        Ok(())
    }
}


impl ProcessorState {
    fn new(hartid: usize, config: ProcessorCfg, clint: Option<IrqVec>, plic: Option<IrqVec>) -> ProcessorState {
        let privilege = PrivilegeStates::new(&config);
        let mut state = ProcessorState {
            hartid,
            config,
            privilege,
            xreg: [0 as RegT; 32],
            extensions: unsafe {
                let mut arr: MaybeUninit<[Extension; 26]> = MaybeUninit::uninit();
                for i in 0..26 {
                    (arr.as_mut_ptr() as *mut Extension).add(i).write(Extension::InvalidExtension);
                }
                arr.assume_init()
            },
            pc: 0,
            next_pc: 0,
            ir: 0,
            insns_cnt:Rc::new(RefCell::new(0)),
            clint,
            plic,
            wfi: false,
        };
        state.add_extension().expect("add extension error!");
        state.privilege.delegate_insns_cnt(state.insns_cnt());
        state
    }

    fn reset(&mut self, start_address: u64) -> Result<(), String> {
        if self.config.xlen == XLen::X32 && start_address.leading_zeros() < 32 {
            return Err(format!("cpu{}:invalid start addr {:#x} when xlen == X32!", self.hartid, start_address));
        }
        self.xreg = [0 as RegT; 32];
        self.pc = 0;
        self.next_pc = start_address;
        self.ir = 0;
        self.wfi = false;
        let m = self.priv_m();
        if let Some(ref clint) = self.clint {
            //register clint:0:msip, 1:mtip
            m.mip_mut().msip_transform({
                let l = clint.listener(0).unwrap();
                move |_| {
                    l.pending_uncheck() as RegT
                }
            });
            m.mip_mut().mtip_transform({
                let l = clint.listener(1).unwrap();
                move |_| {
                    l.pending_uncheck() as RegT
                }
            });
        }
        if let Some(ref pilc) = self.plic {
            //register plic
            m.mip_mut().meip_transform({
                let l = pilc.listener(0).unwrap();
                move |_| {
                    l.pending_uncheck() as RegT
                }
            });
            m.mip_mut().seip_transform({
                let l = pilc.listener(0).unwrap();
                move |_| {
                    l.pending_uncheck() as RegT
                }
            });
        }
        //hartid
        m.mhartid_mut().set(self.hartid as RegT);
        //extensions config, only f, d can disable
        let mut misa = m.misa_mut();
        for ext in self.config().extensions.iter() {
            match ext {
                'a' => misa.set_a(1),
                'b' => misa.set_b(1),
                'c' => misa.set_c(1),
                'd' => misa.set_d(1),
                'e' => misa.set_e(1),
                'f' => misa.set_f(1),
                'g' => misa.set_g(1),
                'h' => misa.set_h(1),
                'i' => misa.set_i(1),
                'j' => misa.set_j(1),
                'k' => misa.set_k(1),
                'l' => misa.set_l(1),
                'm' => misa.set_m(1),
                'n' => misa.set_n(1),
                'o' => misa.set_o(1),
                'p' => misa.set_p(1),
                'q' => misa.set_q(1),
                'r' => misa.set_r(1),
                's' => misa.set_s(1),
                't' => misa.set_t(1),
                'u' => misa.set_u(1),
                'v' => misa.set_v(1),
                'w' => misa.set_w(1),
                'x' => misa.set_x(1),
                'y' => misa.set_y(1),
                'z' => misa.set_z(1),
                _ => unreachable!()
            }
        }
        //xlen config
        match self.config().xlen {
            XLen::X32 => {
                misa.set_mxl(1);
            }
            XLen::X64 => {
                misa.set_mxl(2);
                m.mstatus_mut().set_uxl(2);
                m.mstatus_mut().set_sxl(2);
            }
        }

        Ok(())
    }

    fn add_extension(&mut self) -> Result<(), String> {
        let exts = self.config().extensions.iter().filter(|&e| { *e != 'i' }).map(|e| { *e }).collect::<Vec<char>>();
        let mut add_one_extension = |id: char| -> Result<(), String>  {
            let ext = Extension::new(self, id)?;
            self.extensions[(id as u8 - 'a' as u8) as usize] = ext;
            Ok(())
        };
        add_one_extension('i')?;
        for ext in exts {
            add_one_extension(ext)?
        }
        Ok(())
    }

    fn extensions(&self) -> &[Extension; 26] {
        &self.extensions
    }

    fn get_extension(&self, id: char) -> &Extension {
        unsafe { self.extensions.get_unchecked((id as u8 - 'a' as u8) as usize) }
        // &self.extensions[(id as u8 - 'a' as u8) as usize]
    }

    fn get_extension_mut(&mut self, id: char) -> &mut Extension {
        unsafe { self.extensions.get_unchecked_mut((id as u8 - 'a' as u8) as usize) }
        // &self.extensions[(id as u8 - 'a' as u8) as usize]
    }

    pub fn wfi(&self) -> bool {
        self.wfi
    }

    pub fn set_wfi(&mut self, value: bool) {
        self.wfi = value
    }

    pub fn isa_string(&self) -> String {
        let exts: String = self.config().extensions.iter().collect();
        format!("rv{}{}", self.config().xlen.len(), exts)
    }

    pub fn config(&self) -> &ProcessorCfg {
        &self.config
    }

    pub fn hartid(&self) -> usize {
        self.hartid
    }

    pub fn csr(&self, id: InsnT) -> Result<RegT, Exception> {
        let trip_id = id & 0xfff;
        self.privilege.csr_privilege_check(trip_id).map_err(|_|{Exception::IllegalInsn(*self.ir())})?;
        if let Some(v) = self.privilege.csr_read(self,trip_id) {
            return Ok(v)
        }
        match self.extensions().iter().find_map(|e| { e.csr_read(self, trip_id) }) {
            Some(v) => Ok(v),
            None => Err(Exception::IllegalInsn(*self.ir()))
        }
    }

    pub fn set_csr(&self, id: InsnT, value: RegT) -> Result<(), Exception> {
        let trip_id = id & 0xfff;
        self.privilege.csr_privilege_check(trip_id).map_err(|_|{Exception::IllegalInsn(*self.ir())})?;
        if self.privilege.csr_write(self,trip_id, value).is_some() {
            return Ok(())
        }
        match self.extensions().iter().find_map(|e| { e.csr_write(self, trip_id, value) }) {
            Some(_) => Ok(()),
            None => Err(Exception::IllegalInsn(*self.ir()))
        }
    }

    pub fn check_extension(&self, ext: char) -> Result<(), Exception> {
        self.privilege.check_extension(ext).map_err(|_|{Exception::IllegalInsn(*self.ir())})
    }
    pub fn priv_m(&self) -> &PrivM {
        self.privilege.m()
    }

    pub fn priv_s(&self) -> Result<&PrivS, Exception> {
        self.privilege.s().ok_or(Exception::IllegalInsn(*self.ir()))
    }

    pub fn privilege(&self) -> &Privilege {
        self.privilege.cur_privilege()
    }

    pub fn trap_enter(&mut self, code:RegT, int_flag:bool, val:RegT) {
        let (pc, privilege) = self.privilege.trap_enter(self, code, int_flag, val);
        self.set_pc(pc);
        self.privilege.set_priv(privilege);
    }

    pub fn trap_return(&mut self, cur_privilege:&Privilege) {
        let (pc, privilege) = self.privilege.trap_return(cur_privilege);
        self.set_pc(pc);
        self.privilege.set_priv(privilege);
    }

    pub fn pending_interrupts(&self) -> RegT {
        self.privilege.pending_interrupts()
    }

    pub fn check_xlen(&self, xlen: XLen) -> Result<(), Exception> {
        if xlen == self.config().xlen {
            Ok(())
        } else {
            Err(Exception::IllegalInsn(*self.ir()))
        }
    }

    pub fn pc(&self) -> &RegT {
        &self.pc
    }

    pub fn set_pc(&mut self, pc: RegT) {
        self.next_pc = pc
    }

    pub fn ir(&self) -> &InsnT {
        &self.ir
    }

    pub fn set_ir(&mut self, ir: InsnT) {
        self.ir = ir
    }

    pub fn next_pc(&self) -> &RegT {
        &self.next_pc
    }

    pub fn insns_cnt(&self) -> &Rc<RefCell<u64>> {
        &self.insns_cnt
    }

    pub fn xreg(&self, id: InsnT) -> &RegT {
        let trip_id = id & 0x1f;
        if trip_id == 0 {
            &0
        } else {
            // self.xreg[trip_id as usize]
            unsafe { self.xreg.get_unchecked(trip_id as usize) }
        }
    }

    pub fn set_xreg(&mut self, id: InsnT, value: RegT) {
        let trip_id = id & 0x1f;
        if trip_id != 0 {
            *unsafe { self.xreg.get_unchecked_mut(trip_id as usize) } = value
            // self.xreg[trip_id as usize] = value
        }
    }
}

pub struct Processor {
    state: ProcessorState,
    mmu: Mmu,
    fetcher: Fetcher,
    load_store: LoadStore,
}

impl Processor {
    pub fn new(hartid: usize, config: ProcessorCfg, bus: &Rc<Bus>, clint: Option<IrqVec>, plic: Option<IrqVec>) -> Processor {
        let state = ProcessorState::new(hartid, config, clint, plic);
        let mmu = Mmu::new(bus);
        let fetcher = Fetcher::new(bus);
        let load_store = LoadStore::new(bus);
        Processor {
            state,
            mmu,
            fetcher,
            load_store,
        }
    }

    pub fn reset(&mut self, start_address: u64) -> Result<(), String> {
        self.state.reset(start_address)?;
        self.load_store().release(self.state());
        self.mmu.flush_tlb();
        self.fetcher.flush_icache();
        Ok(())
    }

    pub fn fetcher(&self) -> &Fetcher {
        &self.fetcher
    }

    pub fn mmu(&self) -> &Mmu {
        &self.mmu
    }

    pub fn load_store(&self) -> &LoadStore {
        &self.load_store
    }

    pub fn state(&self) -> &ProcessorState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ProcessorState {
        &mut self.state
    }

    fn one_insn(&mut self) -> Result<(), Exception> {
        self.state_mut().pc = self.state.next_pc;
        let (ir, inst) = self.fetcher.fetch(self.state(), self.mmu())?;
        self.state.ir = ir;
        match inst.execute(self) {
            Ok(_) => {
                *self.state.insns_cnt.deref().borrow_mut() += 1;
                Ok(())
            }
            Err(e) => {
                if e.executed() {
                    *self.state.insns_cnt.deref().borrow_mut() += 1;
                }
                Err(e)
            }
        }
    }

    fn take_interrupt(&self) -> Result<(), Interrupt> {
        const MEIP: RegT = 1 << 11;
        const MSIP: RegT = 1 << 3;
        const MTIP: RegT = 1 << 7;
        const SEIP: RegT = 1 << 9;
        const SSIP: RegT = 1 << 1;
        const STIP: RegT = 1 << 5;

        let interrupts = self.state().pending_interrupts();
        if interrupts == 0 {
            Ok(())
        } else {
            // MEI > MSI > MTI > SEI > SSI > STI
            if interrupts & MEIP != 0 {
                return Err(Interrupt::MEInt);
            } else if interrupts & MSIP != 0 {
                return Err(Interrupt::MSInt);
            } else if interrupts & MTIP != 0 {
                return Err(Interrupt::MTInt);
            } else if interrupts & SEIP != 0 {
                return Err(Interrupt::SEInt);
            } else if interrupts & SSIP != 0 {
                return Err(Interrupt::SSInt);
            } else if interrupts & STIP != 0 {
                return Err(Interrupt::STInt);
            } else {
                unreachable!()
            }
        }
    }

    fn handle_trap(&mut self, trap: Trap) {
        let (int_flag, code, tval) = match trap {
            Trap::Exception(e) => (false, e.code(), e.tval()),
            Trap::Interrupt(i) => (true, i.code(), i.tval()),
        };
        self.state_mut().trap_enter(code, int_flag, tval);
        self.mmu().flush_tlb();
        self.fetcher().flush_icache();
    }

    fn execute_one(&mut self) -> Result<(), Trap> {
        self.take_interrupt()?;
        self.one_insn()?;
        Ok(())
    }

    fn one_step(&mut self) {
        if self.state().wfi() {
            let m = self.state().priv_m();
            if m.mip().get() & m.mie().get() == 0 {
                return;
            } else {
                self.state_mut().set_wfi(false)
            }
        }
        if let Err(trap) = self.execute_one() {
            self.handle_trap(trap)
        }
    }

    pub fn step(&mut self, n: usize) {
        assert!(n > 0);

        for _ in 0..n {
            self.one_step()
        }

        for ext in self.state().extensions().iter() {
            ext.step_cb(self)
        }
    }

    pub fn step_with_debug<O: Write>(&mut self, n: usize, log: &mut O, trace_all: bool) -> Result<(), String> {
        assert!(n > 0);

        for _ in 0..n {
            self.one_step();
            if trace_all {
                log.write_all((self.state.trace() + "\n").as_bytes()).map_err(|e| { e.to_string() })?;
            }
        }

        for ext in self.state().extensions().iter() {
            ext.step_cb(self)
        }
        if !trace_all {
            log.write_all((self.state.trace() + "\n").as_bytes()).map_err(|e| { e.to_string() })?;
        }
        log.write_all(self.state.to_string().as_bytes()).map_err(|e| { e.to_string() })
    }
}
