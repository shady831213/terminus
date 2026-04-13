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

/// A wrapper around TcpStream that logs all GDB RSP packet data to stderr.
///
/// This enables packet-level debugging of the GDB Remote Serial Protocol
/// exchange between the target and GDB client. All bytes written to and
/// read from the connection are logged, making it possible to trace exactly
/// what packets are being sent/received.
pub struct LoggingConnection {
    inner: TcpStream,
    /// Buffer for accumulating outgoing packet data for logging.
    /// GDB RSP packets start with '$' and end with '#XX' (checksum).
    tx_buf: Vec<u8>,
    /// Whether we're currently inside a packet being written.
    in_tx_packet: bool,
    /// Buffer for accumulating incoming packet data for logging.
    rx_buf: Vec<u8>,
    /// Whether we're currently inside a packet being read.
    in_rx_packet: bool,
}

impl LoggingConnection {
    pub fn new(stream: TcpStream) -> Self {
        LoggingConnection {
            inner: stream,
            tx_buf: Vec::new(),
            in_tx_packet: false,
            rx_buf: Vec::new(),
            in_rx_packet: false,
        }
    }

    /// Flush and log any accumulated outgoing packet data.
    fn flush_tx_log(&mut self) {
        if !self.tx_buf.is_empty() {
            let s = String::from_utf8_lossy(&self.tx_buf);
            eprintln!("[GDB TX] {}", s);
            self.tx_buf.clear();
            self.in_tx_packet = false;
        }
    }

    /// Flush and log any accumulated incoming packet data.
    fn flush_rx_log(&mut self) {
        if !self.rx_buf.is_empty() {
            let s = String::from_utf8_lossy(&self.rx_buf);
            eprintln!("[GDB RX] {}", s);
            self.rx_buf.clear();
            self.in_rx_packet = false;
        }
    }

    /// Process an outgoing byte for logging purposes.
    fn log_tx_byte(&mut self, byte: u8) {
        match byte {
            b'$' => {
                // Start of a new packet - flush any previous incomplete data
                self.flush_tx_log();
                self.tx_buf.push(byte);
                self.in_tx_packet = true;
            }
            b'#' if self.in_tx_packet => {
                // End of packet data, checksum follows
                self.tx_buf.push(byte);
                // Don't flush yet - checksum bytes follow
            }
            b'+' | b'-' if !self.in_tx_packet => {
                // Ack/Nack outside of a packet
                self.flush_tx_log();
                eprintln!("[GDB TX] {}", byte as char);
            }
            0x03 if !self.in_tx_packet => {
                // Ctrl-C interrupt outside of a packet
                self.flush_tx_log();
                eprintln!("[GDB TX] <Ctrl-C>");
            }
            _ => {
                self.tx_buf.push(byte);
                // If we see a checksum (2 hex chars after '#'), flush
                if self.in_tx_packet
                    && self.tx_buf.len() >= 3
                    && self.tx_buf[self.tx_buf.len() - 3] == b'#'
                {
                    // We have '#' + 2 hex chars = complete packet
                    self.flush_tx_log();
                }
            }
        }
    }

    /// Process an incoming byte for logging purposes.
    fn log_rx_byte(&mut self, byte: u8) {
        match byte {
            b'$' => {
                // Start of a new packet - flush any previous incomplete data
                self.flush_rx_log();
                self.rx_buf.push(byte);
                self.in_rx_packet = true;
            }
            b'#' if self.in_rx_packet => {
                // End of packet data, checksum follows
                self.rx_buf.push(byte);
            }
            b'+' | b'-' if !self.in_rx_packet => {
                // Ack/Nack outside of a packet
                self.flush_rx_log();
                eprintln!("[GDB RX] {}", byte as char);
            }
            0x03 if !self.in_rx_packet => {
                // Ctrl-C interrupt outside of a packet
                self.flush_rx_log();
                eprintln!("[GDB RX] <Ctrl-C>");
            }
            _ => {
                self.rx_buf.push(byte);
                // If we see a checksum (2 hex chars after '#'), flush
                if self.in_rx_packet
                    && self.rx_buf.len() >= 3
                    && self.rx_buf[self.rx_buf.len() - 3] == b'#'
                {
                    self.flush_rx_log();
                }
            }
        }
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
        self.flush_tx_log();
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

pub struct GdbEventLoop<C>(std::marker::PhantomData<C>);

impl<C> GdbEventLoop<C> {
    pub fn new() -> Self {
        GdbEventLoop(std::marker::PhantomData)
    }
}

impl<C> run_blocking::BlockingEventLoop for GdbEventLoop<C>
where
    C: Connection<Error = std::io::Error> + ConnectionExt<Error = std::io::Error>,
{
    type Target = GdbTarget;
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
                eprintln!("[GDB stepi] PC before step: 0x{:016x}", pc_before);
                let reason = target.processor_mut().step(1);
                let pc_after = *target.processor().state().next_pc();
                eprintln!(
                    "[GDB stepi] Step completed: PC 0x{:016x} -> 0x{:016x}, reason: {:?}",
                    pc_before, pc_after, reason
                );
                let stop_reason = match reason {
                    Some(DebugStopReason::Breakpoint(addr)) => {
                        eprintln!("[GDB stepi] Hit breakpoint at 0x{:016x}", addr);
                        SingleThreadStopReason::SwBreak(())
                    }
                    Some(DebugStopReason::StepComplete) | None => {
                        eprintln!(
                            "[GDB stepi] Sending DoneStep (S05 SIGTRAP) for PC 0x{:016x}",
                            pc_after
                        );
                        SingleThreadStopReason::DoneStep
                    }
                    Some(DebugStopReason::Halted) => {
                        eprintln!("[GDB stepi] Target halted at PC 0x{:016x}", pc_after);
                        SingleThreadStopReason::Terminated(Signal::SIGSTOP)
                    }
                };
                eprintln!("[GDB stepi] Stop reply: {:?}", stop_reason);
                let _ = conn.flush();
                Ok(run_blocking::Event::TargetStopped(stop_reason))
            }
            ExecMode::Continue => {
                eprintln!("[GDB continue] Starting continue mode...");
                let mut cycles = 0;
                loop {
                    if cycles % BATCH_SIZE == 0 {
                        match ConnectionExt::peek(conn) {
                            Ok(Some(byte)) => {
                                eprintln!(
                                    "[GDB continue] Incoming data byte: 0x{:02x} ('{}')",
                                    byte,
                                    if byte.is_ascii_graphic() {
                                        byte as char
                                    } else {
                                        '.'
                                    }
                                );
                                return Ok(run_blocking::Event::IncomingData(byte));
                            }
                            Ok(None) => {}
                            Err(e) => {
                                eprintln!("[GDB continue] Connection peek error: {}", e);
                                return Err(run_blocking::WaitForStopReasonError::Connection(e));
                            }
                        }
                    }
                    cycles += 1;

                    let reason = target.processor_mut().step(1);
                    if let Some(debug_reason) = reason {
                        let pc = *target.processor().state().next_pc();
                        eprintln!(
                            "[GDB continue] Stopped at PC 0x{:016x}, reason: {:?}",
                            pc, debug_reason
                        );
                        let stop_reason = match debug_reason {
                            DebugStopReason::Breakpoint(addr) => {
                                eprintln!("[GDB continue] Hit breakpoint at 0x{:016x}", addr);
                                SingleThreadStopReason::SwBreak(())
                            }
                            DebugStopReason::StepComplete => {
                                eprintln!(
                                    "[GDB continue] Sending DoneStep (S05 SIGTRAP) for PC 0x{:016x}",
                                    pc
                                );
                                SingleThreadStopReason::DoneStep
                            }
                            DebugStopReason::Halted => {
                                eprintln!("[GDB continue] Target halted at PC 0x{:016x}", pc);
                                SingleThreadStopReason::Terminated(Signal::SIGSTOP)
                            }
                        };
                        eprintln!("[GDB continue] Stop reply: {:?}", stop_reason);
                        let _ = conn.flush();
                        return Ok(run_blocking::Event::TargetStopped(stop_reason));
                    }
                    if cycles % 100000 == 0 {
                        eprintln!("[GDB continue] cycles={}", cycles);
                    }
                }
            }
        }
    }

    fn on_interrupt(
        _target: &mut Self::Target,
    ) -> Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        eprintln!("[GDB] Ctrl-C interrupt received, sending SIGINT");
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT)))
    }
}
