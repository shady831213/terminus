use crate::gdb::target::{ExecMode, GdbTarget};
use crate::processor::DebugStopReason;
use gdbstub::common::Signal;
use gdbstub::stub::run_blocking;
use gdbstub::stub::SingleThreadStopReason;
use gdbstub::target::Target;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub fn wait_for_gdb_connection(addr: &str) -> Result<TcpStream, std::io::Error> {
    let listener = TcpListener::bind(addr)?;
    eprintln!("Waiting for GDB connection on {}...", addr);
    let (stream, peer) = listener.accept()?;
    eprintln!("GDB connected from {}", peer);
    stream.set_read_timeout(Some(Duration::from_millis(10)))?;
    Ok(stream)
}

const BATCH_SIZE: usize = 1024;

pub struct GdbEventLoop;

impl run_blocking::BlockingEventLoop for GdbEventLoop {
    type Target = GdbTarget;
    type Connection = TcpStream;
    type StopReason = SingleThreadStopReason<u64>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        run_blocking::Event<Self::StopReason>,
        run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as gdbstub::conn::Connection>::Error,
        >,
    > {
        target.processor_mut().state_mut().set_debug_mode(true);

        match target.exec_mode() {
            ExecMode::Step => {
                eprintln!("Executing step...");
                let reason = target.processor_mut().step(1);
                eprintln!("Step completed with reason: {:?}", reason);
                let stop_reason = match reason {
                    Some(DebugStopReason::Breakpoint(_addr)) => SingleThreadStopReason::SwBreak(()),
                    Some(DebugStopReason::StepComplete) | None => SingleThreadStopReason::DoneStep,
                    Some(DebugStopReason::Halted) => {
                        SingleThreadStopReason::Terminated(Signal::SIGSTOP)
                    }
                };
                eprintln!("Converted to stop_reason: {:?}", stop_reason);
                Ok(run_blocking::Event::TargetStopped(stop_reason))
            }
            ExecMode::Continue => {
                eprintln!("Starting continue mode...");
                let mut cycles = 0;
                loop {
                    if cycles % BATCH_SIZE == 0 {
                        match gdbstub::conn::ConnectionExt::peek(conn) {
                            Ok(Some(byte)) => {
                                eprintln!("Continue loop: incoming data byte: {}", byte);
                                return Ok(run_blocking::Event::IncomingData(byte));
                            }
                            Ok(None) => {}
                            Err(e) => {
                                eprintln!("Continue loop: connection error: {}", e);
                                return Err(run_blocking::WaitForStopReasonError::Connection(e));
                            }
                        }
                    }
                    cycles += 1;

                    let reason = target.processor_mut().step(1);
                    if let Some(debug_reason) = reason {
                        eprintln!(
                            "Continue loop: step returned debug_reason: {:?}",
                            debug_reason
                        );
                        let stop_reason = match debug_reason {
                            DebugStopReason::Breakpoint(_addr) => {
                                SingleThreadStopReason::SwBreak(())
                            }
                            DebugStopReason::StepComplete => SingleThreadStopReason::DoneStep,
                            DebugStopReason::Halted => {
                                SingleThreadStopReason::Terminated(Signal::SIGSTOP)
                            }
                        };
                        eprintln!("Continue loop: converted to stop_reason: {:?}", stop_reason);
                        return Ok(run_blocking::Event::TargetStopped(stop_reason));
                    }
                    if cycles % 10000 == 0 {
                        eprintln!("Continue loop: cycles={}", cycles);
                    }
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
