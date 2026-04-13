use crate::processor::Processor;
use gdbstub::common::Signal;
use gdbstub::target;
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadResumeOps, SingleThreadSingleStep,
    SingleThreadSingleStepOps,
};
use gdbstub::target::ext::breakpoints::{
    Breakpoints, BreakpointsOps, SwBreakpoint, SwBreakpointOps,
};
use gdbstub::target::{Target, TargetError, TargetResult};
use gdbstub_arch::riscv::reg::RiscvCoreRegs;
use gdbstub_arch::riscv::Riscv64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecMode {
    Continue,
    Step,
}

pub struct GdbTarget {
    processor: *mut Processor,
    exec_mode: ExecMode,
}

unsafe impl Send for GdbTarget {}
unsafe impl Sync for GdbTarget {}

impl GdbTarget {
    pub fn new(processor: &mut Processor) -> Self {
        GdbTarget {
            processor: processor as *mut Processor,
            exec_mode: ExecMode::Continue,
        }
    }

    pub fn exec_mode(&self) -> ExecMode {
        self.exec_mode
    }

    pub fn processor(&self) -> &Processor {
        unsafe { &*self.processor }
    }

    pub fn processor_mut(&mut self) -> &mut Processor {
        unsafe { &mut *self.processor }
    }
}

impl Target for GdbTarget {
    type Arch = Riscv64;
    type Error = String;

    #[inline(always)]
    fn base_ops(&mut self) -> target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        target::ext::base::BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn guard_rail_implicit_sw_breakpoints(&self) -> bool {
        true
    }
}

impl SingleThreadBase for GdbTarget {
    fn read_registers(&mut self, regs: &mut RiscvCoreRegs<u64>) -> TargetResult<(), Self> {
        let state = self.processor().state();
        for i in 0..32 {
            regs.x[i] = *state.xreg(i as u32);
        }
        regs.pc = *state.pc();
        Ok(())
    }

    fn write_registers(&mut self, regs: &RiscvCoreRegs<u64>) -> TargetResult<(), Self> {
        let state = self.processor_mut().state_mut();
        for i in 0..32 {
            state.set_xreg(i as u32, regs.x[i]);
        }
        state.set_pc(regs.pc);
        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<usize, Self> {
        let proc = self.processor();
        let ls = proc.load_store();
        let mmu = proc.mmu();
        let state = proc.state();
        for (i, byte) in data.iter_mut().enumerate() {
            let addr = start_addr.wrapping_add(i as u64);
            let mut val: u8 = 0;
            match ls.load_byte(state, &addr, &mut val, mmu) {
                Ok(()) => *byte = val,
                Err(_) => {
                    if i == 0 {
                        return Err(TargetError::NonFatal);
                    }
                    return Ok(i);
                }
            }
        }
        Ok(data.len())
    }

    fn write_addrs(&mut self, start_addr: u64, data: &[u8]) -> TargetResult<(), Self> {
        let proc = self.processor();
        let ls = proc.load_store();
        let mmu = proc.mmu();
        let state = proc.state();
        for (i, byte) in data.iter().enumerate() {
            let addr = start_addr.wrapping_add(i as u64);
            match ls.store_byte(state, &addr, byte, mmu) {
                Ok(()) => {}
                Err(_) => return Err(TargetError::NonFatal),
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn support_single_register_access(
        &mut self,
    ) -> Option<target::ext::base::single_register_access::SingleRegisterAccessOps<'_, (), Self>>
    {
        None
    }
}

impl SingleThreadResume for GdbTarget {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.exec_mode = ExecMode::Continue;
        Ok(())
    }

    #[inline(always)]
    fn support_single_step(&mut self) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadSingleStep for GdbTarget {
    fn step(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.exec_mode = ExecMode::Step;
        Ok(())
    }
}

impl Breakpoints for GdbTarget {
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for GdbTarget {
    fn add_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        self.processor_mut().state_mut().add_sw_breakpoint(addr);
        Ok(true)
    }

    fn remove_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        self.processor_mut().state_mut().remove_sw_breakpoint(addr);
        Ok(true)
    }
}
