use crate::gdb::target::{ExecMode, GdbTarget};
use crate::processor::DebugStopReason;
use gdbstub::common::Signal;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::run_blocking;
use gdbstub::stub::SingleThreadStopReason;
use gdbstub::target::Target;
use std::io::{Read, Write};
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

struct PacketLogState {
    buf: Vec<u8>,
    in_packet: bool,
}

impl PacketLogState {
    fn new() -> Self {
        PacketLogState {
            buf: Vec::new(),
            in_packet: false,
        }
    }

    fn flush(&mut self, verbose: bool, label: &str) {
        if !verbose || self.buf.is_empty() {
            self.buf.clear();
            self.in_packet = false;
            return;
        }
        let s = String::from_utf8_lossy(&self.buf);
        eprintln!("[GDB {}] {}", label, s);
        self.buf.clear();
        self.in_packet = false;
    }

    fn log_byte(&mut self, byte: u8, verbose: bool, label: &str) {
        if !verbose {
            return;
        }
        match byte {
            b'$' => {
                self.flush(verbose, label);
                self.buf.push(byte);
                self.in_packet = true;
            }
            b'#' if self.in_packet => {
                self.buf.push(byte);
            }
            b'+' | b'-' if !self.in_packet => {
                self.flush(verbose, label);
                eprintln!("[GDB {}] {}", label, byte as char);
            }
            0x03 if !self.in_packet => {
                self.flush(verbose, label);
                eprintln!("[GDB {}] <Ctrl-C>", label);
            }
            _ => {
                self.buf.push(byte);
                if self.in_packet && self.buf.len() >= 3 && self.buf[self.buf.len() - 3] == b'#' {
                    self.flush(verbose, label);
                }
            }
        }
    }
}

/// A wrapper around TcpStream that logs all GDB RSP packet data to stderr.
///
/// This enables packet-level debugging of the GDB Remote Serial Protocol
/// exchange between the target and GDB client. All bytes written to and
/// read from the connection are logged, making it possible to trace exactly
/// what packets are being sent/received.
pub struct LoggingConnection {
    inner: TcpStream,
    tx: PacketLogState,
    rx: PacketLogState,
    verbose: bool,
}

impl LoggingConnection {
    pub fn new(stream: TcpStream) -> Self {
        LoggingConnection {
            inner: stream,
            tx: PacketLogState::new(),
            rx: PacketLogState::new(),
            verbose: false,
        }
    }

    pub fn with_verbose(stream: TcpStream, verbose: bool) -> Self {
        LoggingConnection {
            inner: stream,
            tx: PacketLogState::new(),
            rx: PacketLogState::new(),
            verbose,
        }
    }

    fn log_tx_byte(&mut self, byte: u8) {
        self.tx.log_byte(byte, self.verbose, "TX");
    }

    fn log_rx_byte(&mut self, byte: u8) {
        self.rx.log_byte(byte, self.verbose, "RX");
    }
}

impl Connection for LoggingConnection {
    type Error = std::io::Error;

    fn write(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.log_tx_byte(byte);
        Write::write_all(&mut self.inner, &[byte])
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        for &byte in buf {
            self.log_tx_byte(byte);
        }
        Write::write_all(&mut self.inner, buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush(self.verbose, "TX");
        Write::flush(&mut self.inner)
    }

    fn on_session_start(&mut self) -> Result<(), Self::Error> {
        eprintln!("[GDB] Session started - packet tracing enabled");
        self.inner.set_nodelay(true)
    }
}

impl ConnectionExt for LoggingConnection {
    fn read(&mut self) -> Result<u8, Self::Error> {
        self.inner.set_nonblocking(false)?;
        let mut buf = [0u8];
        loop {
            match Read::read_exact(&mut self.inner, &mut buf) {
                Ok(_) => {
                    self.log_rx_byte(buf[0]);
                    return Ok(buf[0]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // EAGAIN: Socket may still be in non-blocking mode from a
                    // prior peek() call. Yield and retry — set_nonblocking(false)
                    // should take effect on the next attempt.
                    std::thread::yield_now();
                    continue;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Read timeout (10ms): No data available yet, retry.
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    fn peek(&mut self) -> Result<Option<u8>, Self::Error> {
        self.inner.set_nonblocking(true)?;
        let mut buf = [0u8];
        let result = match TcpStream::peek(&self.inner, &mut buf) {
            Ok(_) => {
                // Don't log peeked bytes - they'll be logged when actually read
                Ok(Some(buf[0]))
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        };
        // Reset to blocking mode so subsequent read() calls don't hit EAGAIN.
        // Ignore the error here — read() will also set_nonblocking(false) and
        // handle any remaining EAGAIN via its retry loop.
        let _ = self.inner.set_nonblocking(false);
        result
    }
}

fn map_debug_stop_reason(
    reason: Option<DebugStopReason>,
    pc: u64,
    context: &str,
    verbose: bool,
) -> SingleThreadStopReason<u64> {
    match reason {
        Some(DebugStopReason::Breakpoint(addr)) => {
            if verbose {
                eprintln!("[GDB {}] Hit breakpoint at 0x{:016x}", context, addr);
            }
            SingleThreadStopReason::SwBreak(())
        }
        Some(DebugStopReason::StepComplete) | None => {
            if verbose {
                eprintln!(
                    "[GDB {}] Sending DoneStep (S05 SIGTRAP) for PC 0x{:016x}",
                    context, pc
                );
            }
            SingleThreadStopReason::DoneStep
        }
        Some(DebugStopReason::Halted) => {
            if verbose {
                eprintln!("[GDB {}] Target halted at PC 0x{:016x}", context, pc);
            }
            SingleThreadStopReason::Terminated(Signal::SIGSTOP)
        }
    }
}

pub struct GdbEventLoop<'a, C, const VERBOSE: bool>(std::marker::PhantomData<(&'a (), C)>);

impl<'a, C, const VERBOSE: bool> GdbEventLoop<'a, C, VERBOSE> {
    pub fn new() -> Self {
        GdbEventLoop(std::marker::PhantomData)
    }
}

impl<'a, C, const VERBOSE: bool> run_blocking::BlockingEventLoop for GdbEventLoop<'a, C, VERBOSE>
where
    C: Connection<Error = std::io::Error> + ConnectionExt<Error = std::io::Error>,
{
    type Target = GdbTarget<'a>;
    type Connection = C;
    type StopReason = SingleThreadStopReason<u64>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        run_blocking::Event<Self::StopReason>,
        run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as Connection>::Error,
        >,
    > {
        target.processor_mut().state_mut().set_debug_mode(true);

        match target.exec_mode() {
            ExecMode::Step => {
                let pc_before = *target.processor().state().next_pc();
                if VERBOSE {
                    eprintln!("[GDB stepi] PC before step: 0x{:016x}", pc_before);
                }
                let reason = target.processor_mut().step(1);
                let pc_after = *target.processor().state().next_pc();
                if VERBOSE {
                    eprintln!(
                        "[GDB stepi] Step completed: PC 0x{:016x} -> 0x{:016x}, reason: {:?}",
                        pc_before, pc_after, reason
                    );
                }
                let stop_reason = map_debug_stop_reason(reason, pc_after, "stepi", VERBOSE);
                if VERBOSE {
                    eprintln!("[GDB stepi] Stop reply: {:?}", stop_reason);
                }
                let _ = conn.flush();
                Ok(run_blocking::Event::TargetStopped(stop_reason))
            }
            ExecMode::Continue => {
                if VERBOSE {
                    eprintln!("[GDB continue] Starting continue mode...");
                }
                let mut cycles = 0;
                loop {
                    if cycles % BATCH_SIZE == 0 {
                        match ConnectionExt::peek(conn) {
                            Ok(Some(byte)) => {
                                if VERBOSE {
                                    eprintln!(
                                        "[GDB continue] Incoming data byte: 0x{:02x} ('{}')",
                                        byte,
                                        if byte.is_ascii_graphic() {
                                            byte as char
                                        } else {
                                            '.'
                                        }
                                    );
                                }
                                return Ok(run_blocking::Event::IncomingData(byte));
                            }
                            Ok(None) => {}
                            Err(e) => {
                                if VERBOSE {
                                    eprintln!("[GDB continue] Connection peek error: {}", e);
                                }
                                return Err(run_blocking::WaitForStopReasonError::Connection(e));
                            }
                        }
                    }
                    cycles += 1;

                    let reason = target.processor_mut().step(1);
                    if let Some(debug_reason) = reason {
                        let pc = *target.processor().state().next_pc();
                        if VERBOSE {
                            eprintln!(
                                "[GDB continue] Stopped at PC 0x{:016x}, reason: {:?}",
                                pc, debug_reason
                            );
                        }
                        let stop_reason =
                            map_debug_stop_reason(Some(debug_reason), pc, "continue", VERBOSE);
                        if VERBOSE {
                            eprintln!("[GDB continue] Stop reply: {:?}", stop_reason);
                        }
                        let _ = conn.flush();
                        return Ok(run_blocking::Event::TargetStopped(stop_reason));
                    }
                    if cycles % 100000 == 0 && VERBOSE {
                        eprintln!("[GDB continue] cycles={}", cycles);
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
