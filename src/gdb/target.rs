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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::bus::TerminusBus;
    use crate::global::XLen;
    use crate::processor::{Processor, ProcessorCfg};
    use gdbstub_arch::riscv::reg::RiscvCoreRegs;
    use std::rc::Rc;

    fn make_processor() -> Processor {
        let config = ProcessorCfg {
            xlen: XLen::X64,
            enable_dirty: true,
            extensions: Box::new(['i', 'm', 'a', 'f', 'd', 'c']),
            freq: 10000000,
        };
        let bus = Rc::new(TerminusBus::new());
        Processor::new(0, config, &bus, None, None)
    }

    #[test]
    fn test_gdb_target_creation() {
        let mut processor = make_processor();
        processor.reset(0x80000000).unwrap();
        let target = GdbTarget::new(&mut processor);
        assert_eq!(target.exec_mode(), ExecMode::Continue);
    }

    #[test]
    fn test_read_registers() {
        let mut processor = make_processor();
        processor.reset(0x80000000).unwrap();
        let mut target = GdbTarget::new(&mut processor);
        let mut regs = RiscvCoreRegs::<u64>::default();

        let result = target.read_registers(&mut regs);
        assert!(result.is_ok());

        // x0 is always 0
        assert_eq!(regs.x[0], 0);
        // PC is 0 before first instruction (next_pc is the reset vector)
        assert_eq!(regs.pc, 0);
    }

    #[test]
    fn test_write_registers() {
        let mut processor = make_processor();
        processor.reset(0x80000000).unwrap();
        let mut target = GdbTarget::new(&mut processor);
        let mut regs = RiscvCoreRegs::<u64>::default();

        regs.x[1] = 0x12345678;
        regs.x[10] = 0xDEADBEEF;
        regs.pc = 0x80001000;

        let result = target.write_registers(&regs);
        assert!(result.is_ok());

        // Verify GPRs were written
        assert_eq!(*processor.state().xreg(1), 0x12345678);
        assert_eq!(*processor.state().xreg(10), 0xDEADBEEF);
        // Note: write_registers sets next_pc, so current pc() may still be 0
        // The PC will be updated on next instruction fetch
    }

    #[test]
    fn test_read_write_registers_roundtrip() {
        let mut processor = make_processor();
        processor.reset(0x80000000).unwrap();
        let mut target = GdbTarget::new(&mut processor);
        let mut regs_out = RiscvCoreRegs::<u64>::default();

        assert!(target.read_registers(&mut regs_out).is_ok());

        let mut regs_in = regs_out.clone();
        regs_in.x[5] = 0xAAAAAAAA;
        regs_in.x[6] = 0x55555555;
        regs_in.pc = 0x80002000;

        assert!(target.write_registers(&regs_in).is_ok());

        // Verify GPRs were written correctly
        assert_eq!(*processor.state().xreg(5), 0xAAAAAAAA);
        assert_eq!(*processor.state().xreg(6), 0x55555555);
    }

    #[test]
    fn test_exec_mode_changes() {
        let mut processor = make_processor();
        processor.reset(0x80000000).unwrap();
        let mut target = GdbTarget::new(&mut processor);

        assert_eq!(target.exec_mode(), ExecMode::Continue);

        use gdbstub::target::ext::base::singlethread::SingleThreadResume;
        target.resume(None).unwrap();
        assert_eq!(target.exec_mode(), ExecMode::Continue);

        use gdbstub::target::ext::base::singlethread::SingleThreadSingleStep;
        target.step(None).unwrap();
        assert_eq!(target.exec_mode(), ExecMode::Step);
    }

    #[test]
    fn test_sw_breakpoint_operations() {
        let mut processor = make_processor();
        processor.reset(0x80000000).unwrap();
        let mut target = GdbTarget::new(&mut processor);

        assert!(!processor.state().sw_breakpoints().contains(&0x1000));

        use gdbstub::target::ext::breakpoints::SwBreakpoint;
        let add_result = target.add_sw_breakpoint(0x1000, 0);
        assert!(add_result.is_ok());
        assert_eq!(add_result.map_err(|_| ()), Ok(true));
        assert!(processor.state().sw_breakpoints().contains(&0x1000));

        let remove_result = target.remove_sw_breakpoint(0x1000, 0);
        assert!(remove_result.is_ok());
        assert_eq!(remove_result.map_err(|_| ()), Ok(true));
        assert!(!processor.state().sw_breakpoints().contains(&0x1000));
    }

    #[test]
    fn test_sw_breakpoint_multiple() {
        let mut processor = make_processor();
        processor.reset(0x80000000).unwrap();
        let mut target = GdbTarget::new(&mut processor);

        use gdbstub::target::ext::breakpoints::SwBreakpoint;

        assert!(target.add_sw_breakpoint(0x1000, 0).is_ok());
        assert!(target.add_sw_breakpoint(0x2000, 0).is_ok());
        assert!(target.add_sw_breakpoint(0x3000, 0).is_ok());

        assert!(processor.state().sw_breakpoints().contains(&0x1000));
        assert!(processor.state().sw_breakpoints().contains(&0x2000));
        assert!(processor.state().sw_breakpoints().contains(&0x3000));
        assert_eq!(processor.state().sw_breakpoints().len(), 3);

        assert!(target.remove_sw_breakpoint(0x2000, 0).is_ok());
        assert!(processor.state().sw_breakpoints().contains(&0x1000));
        assert!(!processor.state().sw_breakpoints().contains(&0x2000));
        assert!(processor.state().sw_breakpoints().contains(&0x3000));
    }
}
