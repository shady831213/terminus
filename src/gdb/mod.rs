use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use gdbstub::common::Signal;
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadResumeOps, SingleThreadSingleStep,
    SingleThreadSingleStepOps,
};
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::breakpoints::{
    Breakpoints, BreakpointsOps, SwBreakpoint, SwBreakpointOps,
};
use gdbstub::target::{Target, TargetError, TargetResult};
use gdbstub_arch::riscv::reg::RiscvCoreRegs;

use crate::devices::virtio_console::VirtIOConsoleDevice;
use crate::devices::virtio_net::VirtIONetDevice;
use crate::system::System;

pub struct SimContext {
    pub virtio_console: Rc<VirtIOConsoleDevice>,
    pub virtio_net: Option<Rc<VirtIONetDevice>>,
    pub step_cnt: usize,
    pub single_step: bool,
}

pub struct TerminusTarget<'a> {
    pub sys: &'a mut System,
    pub breakpoints: HashMap<u64, Vec<u8>>,
    pub ctx: SimContext,
}

impl<'a> TerminusTarget<'a> {
    pub fn new(sys: &'a mut System, ctx: SimContext) -> Self {
        Self {
            sys,
            breakpoints: HashMap::new(),
            ctx,
        }
    }

    pub fn breakpoint_addrs(&self) -> HashSet<u64> {
        self.breakpoints.keys().copied().collect()
    }
}

impl<'a> Target for TerminusTarget<'a> {
    type Arch = gdbstub_arch::riscv::Riscv64;
    type Error = String;

    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl<'a> SingleThreadBase for TerminusTarget<'a> {
    fn read_registers(&mut self, regs: &mut RiscvCoreRegs<u64>) -> TargetResult<(), Self> {
        let p = self.sys.processor(0).ok_or(TargetError::NonFatal)?;
        for i in 0..32u32 {
            regs.x[i as usize] = *p.state().xreg(i);
        }
        regs.pc = *p.state().next_pc();
        Ok(())
    }

    fn write_registers(&mut self, regs: &RiscvCoreRegs<u64>) -> TargetResult<(), Self> {
        let p = self.sys.processor(0).ok_or(TargetError::NonFatal)?;
        for i in 1..32u32 {
            p.state_mut().set_xreg(i, regs.x[i as usize]);
        }
        p.state_mut().set_pc(regs.pc);
        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<usize, Self> {
        let p = self.sys.processor(0).ok_or(TargetError::NonFatal)?;
        for (i, byte) in data.iter_mut().enumerate() {
            let va = start_addr + i as u64;
            let pa = p.translate_vaddr(va).unwrap_or(va);
            p.read_mem_physical(pa, std::slice::from_mut(byte))
                .map_err(|_| TargetError::NonFatal)?;
        }
        Ok(data.len())
    }

    fn write_addrs(&mut self, start_addr: u64, data: &[u8]) -> TargetResult<(), Self> {
        let p = self.sys.processor(0).ok_or(TargetError::NonFatal)?;
        for (i, byte) in data.iter().enumerate() {
            let va = start_addr + i as u64;
            let pa = p.translate_vaddr(va).unwrap_or(va);
            p.write_mem_physical(pa, std::slice::from_ref(byte))
                .map_err(|_| TargetError::NonFatal)?;
        }
        Ok(())
    }

    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl<'a> SingleThreadResume for TerminusTarget<'a> {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.ctx.single_step = false;
        Ok(())
    }

    fn support_single_step(&mut self) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl<'a> SingleThreadSingleStep for TerminusTarget<'a> {
    fn step(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.ctx.single_step = true;
        Ok(())
    }
}

impl<'a> Breakpoints for TerminusTarget<'a> {
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl<'a> SwBreakpoint for TerminusTarget<'a> {
    fn add_sw_breakpoint(&mut self, addr: u64, kind: usize) -> TargetResult<bool, Self> {
        let mut saved = vec![0u8; kind];
        {
            let p = self.sys.processor(0).ok_or(TargetError::NonFatal)?;
            for (i, byte) in saved.iter_mut().enumerate() {
                let va = addr + i as u64;
                let pa = p
                    .mmu()
                    .fetch_translate(p.state(), &va, 1)
                    .map_err(|_| TargetError::NonFatal)?;
                p.read_mem_physical(pa, std::slice::from_mut(byte))
                    .map_err(|_| TargetError::NonFatal)?;
            }
        }

        self.breakpoints.insert(addr, saved);

        let ebreak_bytes: &[u8] = if kind == 2 {
            &[0x02, 0x90]
        } else {
            &[0x73, 0x00, 0x10, 0x00]
        };

        let p = self.sys.processor(0).ok_or(TargetError::NonFatal)?;
        for (i, byte) in ebreak_bytes.iter().enumerate() {
            let va = addr + i as u64;
            let pa = p
                .mmu()
                .fetch_translate(p.state(), &va, 1)
                .map_err(|_| TargetError::NonFatal)?;
            p.write_mem_physical(pa, std::slice::from_ref(byte))
                .map_err(|_| TargetError::NonFatal)?;
        }
        p.fetcher().flush_icache();
        Ok(true)
    }

    fn remove_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        let saved = match self.breakpoints.remove(&addr) {
            Some(saved) => saved,
            None => return Ok(false),
        };

        let p = self.sys.processor(0).ok_or(TargetError::NonFatal)?;
        for (i, byte) in saved.iter().enumerate() {
            let va = addr + i as u64;
            let pa = p
                .mmu()
                .fetch_translate(p.state(), &va, 1)
                .map_err(|_| TargetError::NonFatal)?;
            p.write_mem_physical(pa, std::slice::from_ref(byte))
                .map_err(|_| TargetError::NonFatal)?;
        }
        p.fetcher().flush_icache();
        Ok(true)
    }
}
