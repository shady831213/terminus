use std::collections::HashSet;
use std::net::{TcpListener, TcpStream};

use gdbstub::common::Signal;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::run_blocking::{BlockingEventLoop, Event, WaitForStopReasonError};
use gdbstub::stub::{GdbStub, SingleThreadStopReason};
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
use gdbstub_arch::riscv::Riscv64;

use crate::devices::bus::Bus;
use crate::processor::trap::{Exception, Trap};
use crate::system::System;
use terminus_spaceport::EXIT_CTRL;

// ---------------------------------------------------------------------------
// ExecMode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExecMode {
    Run,
    Step,
}

// ---------------------------------------------------------------------------
// GdbTarget
// ---------------------------------------------------------------------------

pub struct GdbTarget<'a> {
    system: &'a mut System,
    current_hart: usize,
    breakpoints: HashSet<u64>,
    exec_mode: ExecMode,
    /// When true, the next step in wait_for_stop_reason will skip the
    /// breakpoint check to allow continuing past a breakpoint that was
    /// just hit.
    step_over_bp: bool,
}

impl<'a> GdbTarget<'a> {
    pub fn new(system: &'a mut System, hartid: usize) -> Self {
        GdbTarget {
            system,
            current_hart: hartid,
            breakpoints: HashSet::new(),
            exec_mode: ExecMode::Run,
            step_over_bp: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Target trait
// ---------------------------------------------------------------------------

impl Target for GdbTarget<'_> {
    type Arch = Riscv64;
    type Error = &'static str;

    #[inline(always)]
    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

// ---------------------------------------------------------------------------
// SingleThreadBase
// ---------------------------------------------------------------------------

impl SingleThreadBase for GdbTarget<'_> {
    fn read_registers(&mut self, regs: &mut RiscvCoreRegs<u64>) -> TargetResult<(), Self> {
        let proc = self
            .system
            .processor(self.current_hart)
            .ok_or(TargetError::Fatal("hart not found"))?;
        let state = proc.state();

        regs.x[0] = 0;
        for i in 1..32 {
            regs.x[i] = *state.xreg(i as u32);
        }
        // Use next_pc: this is the address of the next instruction to execute,
        // which is what GDB expects.  The emulator uses a two-phase PC design:
        //   pc     = last-executed instruction address (stale on first read)
        //   next_pc = next instruction to execute
        regs.pc = *state.next_pc();

        Ok(())
    }

    fn write_registers(&mut self, regs: &RiscvCoreRegs<u64>) -> TargetResult<(), Self> {
        let proc = self
            .system
            .processor(self.current_hart)
            .ok_or(TargetError::Fatal("hart not found"))?;
        let state = proc.state_mut();

        for i in 1..32 {
            state.set_xreg(i as u32, regs.x[i]);
        }
        state.set_pc(regs.pc);

        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<usize, Self> {
        let bus = self.system.bus().clone();

        let mut offset = 0;
        while offset < data.len() {
            let addr = start_addr + offset as u64;
            let remaining = data.len() - offset;

            if remaining >= 8 && addr % 8 == 0 {
                let mut val: u64 = 0;
                if bus.read_u64(&addr, &mut val).is_err() {
                    break;
                }
                data[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
                offset += 8;
            } else if remaining >= 4 && addr % 4 == 0 {
                let mut val: u32 = 0;
                if bus.read_u32(&addr, &mut val).is_err() {
                    break;
                }
                data[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
                offset += 4;
            } else if remaining >= 2 && addr % 2 == 0 {
                let mut val: u16 = 0;
                if bus.read_u16(&addr, &mut val).is_err() {
                    break;
                }
                data[offset..offset + 2].copy_from_slice(&val.to_le_bytes());
                offset += 2;
            } else {
                let mut val: u8 = 0;
                if bus.read_u8(&addr, &mut val).is_err() {
                    break;
                }
                data[offset] = val;
                offset += 1;
            }
        }

        Ok(offset)
    }

    fn write_addrs(&mut self, start_addr: u64, data: &[u8]) -> TargetResult<(), Self> {
        let bus = self.system.bus().clone();

        let mut offset = 0;
        while offset < data.len() {
            let addr = start_addr + offset as u64;
            let remaining = data.len() - offset;

            if remaining >= 8 && addr % 8 == 0 {
                let val = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
                bus.write_u64(&addr, &val)
                    .map_err(|_| TargetError::NonFatal)?;
                offset += 8;
            } else if remaining >= 4 && addr % 4 == 0 {
                let val = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                bus.write_u32(&addr, &val)
                    .map_err(|_| TargetError::NonFatal)?;
                offset += 4;
            } else if remaining >= 2 && addr % 2 == 0 {
                let val = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
                bus.write_u16(&addr, &val)
                    .map_err(|_| TargetError::NonFatal)?;
                offset += 2;
            } else {
                bus.write_u8(&addr, &data[offset])
                    .map_err(|_| TargetError::NonFatal)?;
                offset += 1;
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

// ---------------------------------------------------------------------------
// SingleThreadResume  (support_single_step lives here in 0.7)
// ---------------------------------------------------------------------------

impl SingleThreadResume for GdbTarget<'_> {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.exec_mode = ExecMode::Run;
        Ok(())
    }

    #[inline(always)]
    fn support_single_step(&mut self) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

// ---------------------------------------------------------------------------
// SingleThreadSingleStep
// ---------------------------------------------------------------------------

impl SingleThreadSingleStep for GdbTarget<'_> {
    fn step(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.exec_mode = ExecMode::Step;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Breakpoints + SwBreakpoint
// ---------------------------------------------------------------------------

impl Breakpoints for GdbTarget<'_> {
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for GdbTarget<'_> {
    fn add_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        Ok(self.breakpoints.insert(addr))
    }

    fn remove_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        Ok(self.breakpoints.remove(&addr))
    }
}

// ---------------------------------------------------------------------------
// BlockingEventLoop
// ---------------------------------------------------------------------------

impl<'a> BlockingEventLoop for GdbTarget<'a> {
    type Target = GdbTarget<'a>;
    type Connection = TcpStream;
    type StopReason = SingleThreadStopReason<u64>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        Event<Self::StopReason>,
        WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as Connection>::Error,
        >,
    > {
        let mut cycles: u64 = 0;

        loop {
            // Check for breakpoint hit BEFORE executing — the emulator does
            // not patch memory with EBREAK, so we stop when the PC reaches a
            // registered breakpoint address.
            if !target.step_over_bp {
                let next_pc = {
                    let proc = target
                        .system
                        .processor(target.current_hart)
                        .ok_or(WaitForStopReasonError::Target("hart not found"))?;
                    *proc.state().next_pc()
                };
                if target.breakpoints.contains(&next_pc) {
                    target.step_over_bp = true;
                    return Ok(Event::TargetStopped(SingleThreadStopReason::SwBreak(())));
                }
            }

            // Execute one instruction.
            let stop_after = target.exec_mode == ExecMode::Step;
            let stop_reason: Option<SingleThreadStopReason<u64>> = {
                let proc = target
                    .system
                    .processor(target.current_hart)
                    .ok_or(WaitForStopReasonError::Target("hart not found"))?;

                match proc.step_one_debug() {
                    Some(Trap::Exception(Exception::Breakpoint)) => {
                        Some(SingleThreadStopReason::SwBreak(()))
                    }
                    Some(_other_trap) => None,
                    None => {
                        if stop_after {
                            Some(SingleThreadStopReason::DoneStep)
                        } else {
                            None
                        }
                    }
                }
            };

            // After stepping past a breakpoint, clear the override so normal
            // breakpoint checking resumes.
            target.step_over_bp = false;

            if let Some(reason) = stop_reason {
                return Ok(Event::TargetStopped(reason));
            }

            cycles += 1;
            if cycles % 500 == 0 {
                // Check for host-initiated exit (e.g. HTIF shutdown)
                if let Ok(msg) = EXIT_CTRL.poll() {
                    eprintln!("GDB: program exited ({})", msg);
                    return Ok(Event::TargetStopped(SingleThreadStopReason::Signal(
                        Signal::SIGTERM,
                    )));
                }
                match conn.peek().map_err(WaitForStopReasonError::Connection)? {
                    Some(byte) if byte == 0x03 => {
                        return Ok(Event::TargetStopped(SingleThreadStopReason::Signal(
                            Signal::SIGINT,
                        )));
                    }
                    _ => {}
                }
            }
        }
    }

    fn on_interrupt(
        _target: &mut Self::Target,
    ) -> Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT)))
    }
}

// ---------------------------------------------------------------------------
// run_gdb_session - public entry-point (single-threaded, blocking)
// ---------------------------------------------------------------------------

/// Start a GDB remote debugging session, driving the simulation on the current
/// thread.  Blocks until GDB disconnects or an error occurs.
pub fn run_gdb_session(system: &mut System, port: u16, hartid: usize) {
    let listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("GDB: failed to bind port {}: {}", port, e);
            return;
        }
    };
    eprintln!("GDB: listening on port {}", port);

    let (stream, addr) = match listener.accept() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("GDB: accept failed: {}", e);
            return;
        }
    };
    eprintln!("GDB: connection from {}", addr);

    stream.set_nodelay(true).ok();
    stream.set_nonblocking(true).ok();

    let mut target = GdbTarget::new(system, hartid);
    let gdb = GdbStub::new(stream);
    match gdb.run_blocking::<GdbTarget>(&mut target) {
        Ok(reason) => eprintln!("GDB: disconnected ({:?})", reason),
        Err(e) => eprintln!("GDB: error: {}", e),
    }
}
