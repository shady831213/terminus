//! GDB Integration Tests
//!
//! These tests verify GDB remote debugging functionality.
//! Tests that require `riscv64-unknown-elf-gdb` are marked with `#[ignore]`.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

/// GDB RSP (Remote Serial Protocol) packet utilities
mod rsp {
    /// Calculate the checksum for a GDB RSP packet
    pub fn calc_checksum(data: &str) -> u8 {
        data.bytes().fold(0u8, |acc, b| acc.wrapping_add(b))
    }

    /// Create a properly formatted GDB RSP packet
    pub fn make_packet(data: &str) -> String {
        let checksum = calc_checksum(data);
        format!("${}#{:02x}", data, checksum)
    }

    /// Acknowledge packet (send '+' response)
    pub fn ack() -> &'static [u8] {
        b"+"
    }

    /// Not acknowledge packet (send '-' response)
    pub fn nack() -> &'static [u8] {
        b"-"
    }

    /// Escape special characters in RSP binary data
    pub fn escape_data(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        for &b in data {
            match b {
                0x23 => {
                    result.push(0x7d);
                    result.push(0x03);
                } // # -> }#
                0x24 => {
                    result.push(0x7d);
                    result.push(0x04);
                } // $ -> }$
                0x7d => {
                    result.push(0x7d);
                    result.push(0x5d);
                } // } -> }}
                _ => result.push(b),
            }
        }
        result
    }
}

/// Helper to start terminus with GDB server
pub struct TerminusGdbRunner {
    process: Child,
    port: u16,
}

impl TerminusGdbRunner {
    /// Start terminus with the given ELF file and GDB port
    pub fn start(elf_path: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cmd = Command::new("cargo");
        cmd.args(&[
            "run",
            "--",
            elf_path,
            "--gdb",
            &format!("127.0.0.1:{}", port),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());

        let process = cmd.spawn()?;

        // Wait a bit for the server to start
        thread::sleep(Duration::from_millis(500));

        Ok(TerminusGdbRunner { process, port })
    }

    /// Get the port the GDB server is listening on
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Connect to the GDB server
    pub fn connect(&self) -> Result<TcpStream, std::io::Error> {
        let addr = format!("127.0.0.1:{}", self.port);
        TcpStream::connect(&addr)
    }

    /// Kill the terminus process
    pub fn stop(mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Drop for TerminusGdbRunner {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Read a complete GDB RSP packet from the stream
fn read_packet(stream: &mut TcpStream) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 1];
    let mut packet = String::new();

    // Wait for '$'
    loop {
        stream.read_exact(&mut buffer)?;
        if buffer[0] == b'$' {
            break;
        }
    }

    // Read until '#'
    loop {
        stream.read_exact(&mut buffer)?;
        if buffer[0] == b'#' {
            break;
        }
        packet.push(buffer[0] as char);
    }

    // Read 2-byte checksum
    let mut checksum_buf = [0u8; 2];
    stream.read_exact(&mut checksum_buf)?;

    Ok(packet)
}

/// Send a GDB RSP packet and wait for acknowledgment
fn send_packet(stream: &mut TcpStream, data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let packet = rsp::make_packet(data);
    stream.write_all(packet.as_bytes())?;
    stream.flush()?;

    // Read acknowledgment
    let mut ack_buf = [0u8; 1];
    stream.read_exact(&mut ack_buf)?;

    if ack_buf[0] == b'+' {
        // Read response packet
        read_packet(stream)
    } else {
        Err("Negative acknowledgment".into())
    }
}

/// Check if riscv64-unknown-elf-gdb is available
fn gdb_available() -> bool {
    Command::new("riscv64-unknown-elf-gdb")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Skip test if GDB is not available
fn skip_if_no_gdb() {
    if !gdb_available() {
        eprintln!("Skipping test: riscv64-unknown-elf-gdb not available");
        return;
    }
}

#[test]
fn test_gdb_module_compiles() {
    // This test verifies the GDB module compiles correctly
    // It's a basic smoke test that always runs
    use terminus::gdb::{ExecMode, GdbTarget, RiscvRegId, TARGET_DESCRIPTION_XML};

    // Verify TARGET_DESCRIPTION_XML is not empty
    assert!(!TARGET_DESCRIPTION_XML.is_empty());
    assert!(TARGET_DESCRIPTION_XML.contains("riscv:rv64"));

    // Verify ExecMode variants exist
    let _mode = ExecMode::Continue;
    let _mode = ExecMode::Step;
}

#[test]
fn test_reg_id_mapping_comprehensive() {
    use gdbstub::arch::RegId;
    use terminus::gdb::RiscvRegId;

    // Test GPR mapping (0-31)
    for i in 0..32usize {
        let (reg, size) = RiscvRegId::from_raw_id(i).expect(&format!("GPR {} should be valid", i));
        match reg {
            RiscvRegId::Gpr(idx) => assert_eq!(idx as usize, i),
            _ => panic!("Expected GPR for regnum {}", i),
        }
        assert!(size.is_some());
    }

    // Test PC mapping (32)
    let (reg, size) = RiscvRegId::from_raw_id(32).expect("PC should be valid");
    assert!(matches!(reg, RiscvRegId::Pc));
    assert!(size.is_some());

    // Test FPR mapping (33-64)
    for i in 33..=64usize {
        let (reg, size) =
            RiscvRegId::from_raw_id(i).expect(&format!("FPR {} should be valid", i - 33));
        match reg {
            RiscvRegId::Fpr(idx) => assert_eq!(idx as usize, i - 33),
            _ => panic!("Expected FPR for regnum {}", i),
        }
        assert!(size.is_some());
    }

    // Test FCSR mapping (65)
    let (reg, size) = RiscvRegId::from_raw_id(65).expect("FCSR should be valid");
    assert!(matches!(reg, RiscvRegId::Fcsr));
    assert!(size.is_some());

    // Test CSR mapping (66-91)
    for i in 66..=91usize {
        let (reg, _size) = RiscvRegId::from_raw_id(i).expect(&format!("CSR {} should be valid", i));
        assert!(
            matches!(reg, RiscvRegId::Csr(_)),
            "Expected CSR for regnum {}",
            i
        );
    }

    // Test Priv mapping (132)
    let (reg, size) = RiscvRegId::from_raw_id(132).expect("Priv should be valid");
    assert!(matches!(reg, RiscvRegId::Priv));
    assert!(size.is_none()); // Virtual register has no size
}

#[test]
fn test_target_description_complete() {
    use terminus::gdb::TARGET_DESCRIPTION_XML;

    // Verify all required features are present
    assert!(TARGET_DESCRIPTION_XML.contains("org.gnu.gdb.riscv.cpu"));
    assert!(TARGET_DESCRIPTION_XML.contains("org.gnu.gdb.riscv.fpu"));
    assert!(TARGET_DESCRIPTION_XML.contains("org.gnu.gdb.riscv.csr"));
    assert!(TARGET_DESCRIPTION_XML.contains("org.gnu.gdb.riscv.virtual"));

    // Count registers in CPU feature (should be 32 GPR + PC = 33)
    let cpu_feature_start = TARGET_DESCRIPTION_XML
        .find("org.gnu.gdb.riscv.cpu")
        .unwrap();
    let cpu_feature_end = TARGET_DESCRIPTION_XML[cpu_feature_start..]
        .find("</feature>")
        .map(|i| cpu_feature_start + i)
        .unwrap();
    let cpu_feature = &TARGET_DESCRIPTION_XML[cpu_feature_start..cpu_feature_end];
    let cpu_reg_count = cpu_feature.matches("<reg ").count();
    assert_eq!(cpu_reg_count, 33, "CPU feature should have 33 registers");

    // Count registers in FPU feature (should be 32 FPR + fflags + frm + fcsr = 35)
    let fpu_feature_start = TARGET_DESCRIPTION_XML
        .find("org.gnu.gdb.riscv.fpu")
        .unwrap();
    let fpu_feature_end = TARGET_DESCRIPTION_XML[fpu_feature_start..]
        .find("</feature>")
        .map(|i| fpu_feature_start + i)
        .unwrap();
    let fpu_feature = &TARGET_DESCRIPTION_XML[fpu_feature_start..fpu_feature_end];
    let fpu_reg_count = fpu_feature.matches("<reg ").count();
    assert_eq!(fpu_reg_count, 35, "FPU feature should have 35 registers");

    // Count registers in CSR feature (should be 26)
    let csr_feature_start = TARGET_DESCRIPTION_XML
        .find("org.gnu.gdb.riscv.csr")
        .unwrap();
    let csr_feature_end = TARGET_DESCRIPTION_XML[csr_feature_start..]
        .find("</feature>")
        .map(|i| csr_feature_start + i)
        .unwrap();
    let csr_feature = &TARGET_DESCRIPTION_XML[csr_feature_start..csr_feature_end];
    let csr_reg_count = csr_feature.matches("<reg ").count();
    assert_eq!(csr_reg_count, 26, "CSR feature should have 26 registers");
}

// The following tests require an actual GDB server running
// They are marked with #[ignore] and can be run with:
// cargo test --ignored --test gdb_integration

#[test]
#[ignore = "requires terminus binary to be built"]
fn test_gdb_server_starts() {
    // Find a test ELF file
    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        eprintln!("Test ELF not found: {}", test_elf);
        return;
    }

    // Use a random port to avoid conflicts
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // Start terminus with GDB
    let runner = TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus");

    // Give the server time to start
    thread::sleep(Duration::from_millis(1000));

    // Try to connect
    let result = runner.connect();
    assert!(result.is_ok(), "Should be able to connect to GDB server");

    // Clean up
    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_connection_and_handshake() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    // Get available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // Start terminus
    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");

    // Wait for server
    thread::sleep(Duration::from_millis(1000));

    // Connect directly with TCP to test RSP handshake
    let mut stream = runner.connect().expect("Failed to connect to GDB server");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("Failed to set timeout");

    // Send initial packet (qSupported)
    let response = send_packet(&mut stream, "qSupported");
    assert!(response.is_ok(), "Should receive qSupported response");

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_read_registers() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    let mut stream = runner.connect().expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Send halt to stop the target
    let _ = send_packet(&mut stream, "?");

    // Read all registers (g packet)
    let response = send_packet(&mut stream, "g");
    assert!(response.is_ok(), "Should be able to read registers");

    let reg_data = response.unwrap();
    // Should have at least 32 GPRs (8 bytes each = 64 hex chars) + PC (16 hex chars)
    assert!(
        reg_data.len() >= 528,
        "Register data too short: {} chars",
        reg_data.len()
    );

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_memory_read() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    let mut stream = runner.connect().expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Read memory at reset vector (m addr,length)
    // Read 8 bytes from 0x80000000
    let response = send_packet(&mut stream, "m80000000,8");
    assert!(response.is_ok(), "Should be able to read memory");

    let mem_data = response.unwrap();
    // Should be 16 hex characters (8 bytes)
    assert_eq!(
        mem_data.len(),
        16,
        "Memory read should return 16 hex chars, got: {}",
        mem_data.len()
    );

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_step_instruction() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    let mut stream = runner.connect().expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Get initial PC
    let response = send_packet(&mut stream, "g").expect("Failed to read registers");
    // PC is at offset 32 * 16 = 512 hex chars
    let initial_pc = &response[512..528];

    // Step one instruction (s)
    let response = send_packet(&mut stream, "s");
    assert!(response.is_ok(), "Should be able to step");

    // Read registers again
    let response = send_packet(&mut stream, "g").expect("Failed to read registers");
    let new_pc = &response[512..528];

    // PC should have changed after step
    assert_ne!(initial_pc, new_pc, "PC should change after single step");

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_breakpoint() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    let mut stream = runner.connect().expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Set breakpoint at 0x80000004 (Z0,addr,kind)
    let response = send_packet(&mut stream, "Z0,80000004,4");
    assert!(response.is_ok(), "Should be able to set breakpoint");
    assert_eq!(
        response.unwrap(),
        "OK",
        "Breakpoint should be set successfully"
    );

    // Remove breakpoint (z0,addr,kind)
    let response = send_packet(&mut stream, "z0,80000004,4");
    assert!(response.is_ok(), "Should be able to remove breakpoint");
    assert_eq!(
        response.unwrap(),
        "OK",
        "Breakpoint should be removed successfully"
    );

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_continue() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    let mut stream = runner.connect().expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Continue execution (c)
    stream
        .write_all(rsp::make_packet("c").as_bytes())
        .expect("Failed to send continue");
    stream.flush().expect("Failed to flush");

    // Wait a bit for execution
    thread::sleep(Duration::from_millis(100));

    // Send interrupt (Ctrl-C, 0x03)
    stream.write_all(&[0x03]).expect("Failed to send interrupt");
    stream.flush().expect("Failed to flush");

    // Should receive stop reason
    let response = read_packet(&mut stream);
    assert!(
        response.is_ok(),
        "Should receive stop reason after interrupt"
    );

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_target_description() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    let mut stream = runner.connect().expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Request target description
    let response = send_packet(&mut stream, "qXfer:features:read:target.xml:0,1000");
    assert!(
        response.is_ok(),
        "Should be able to read target description"
    );

    let xml_data = response.unwrap();
    assert!(
        xml_data.contains("riscv") || xml_data.contains("org.gnu.gdb.riscv"),
        "Target description should contain RISC-V info"
    );

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_detach() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    let mut stream = runner.connect().expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Send detach (D)
    let response = send_packet(&mut stream, "D");
    assert!(response.is_ok(), "Should be able to detach");

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb and running terminus server"]
fn test_gdb_no_ack_mode() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    let mut stream = runner.connect().expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Enable no-ack mode (QStartNoAckMode)
    let response = send_packet(&mut stream, "QStartNoAckMode");
    assert!(response.is_ok(), "Should be able to enable no-ack mode");

    runner.stop();
}

#[test]
#[ignore = "requires riscv64-unknown-elf-gdb"]
fn test_gdb_binary_with_gdb_cli() {
    skip_if_no_gdb();

    let test_elf = "top_tests/elf/rv64ui-p-add";
    if !Path::new(test_elf).exists() {
        panic!("Test ELF not found: {}", test_elf);
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // Start terminus in background
    let runner =
        TerminusGdbRunner::start(test_elf, port).expect("Failed to start terminus with GDB");
    thread::sleep(Duration::from_millis(1000));

    // Create GDB script
    let gdb_script = format!(
        r#"
set architecture riscv:rv64
target remote 127.0.0.1:{}
info registers
quit
"#,
        port
    );

    // Run GDB with script
    let output = Command::new("riscv64-unknown-elf-gdb")
        .arg("-batch")
        .arg("-x")
        .arg("/dev/stdin")
        .arg(test_elf)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(gdb_script.as_bytes())?;
            }
            child.wait_with_output()
        });

    assert!(output.is_ok(), "GDB should execute without errors");

    let output = output.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("pc") || stdout.contains("PC"),
        "GDB output should contain PC register"
    );

    runner.stop();
}
