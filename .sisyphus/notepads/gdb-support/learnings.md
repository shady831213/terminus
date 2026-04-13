# GDB Support Implementation Notes

## Task 1 Completion Notes (2026-04-13)

### What was done:
- Added gdbstub = "0.7" and gdbstub_arch = "0.3" to Cargo.toml dependencies
- Created src/gdb/ directory with module skeleton
- Created src/gdb/mod.rs with public exports
- Created src/gdb/target.rs with empty GdbTarget struct
- Created src/gdb/event_loop.rs with empty GdbEventLoop struct
- Created src/gdb/reg_id.rs with placeholder RiscvRegId enum (single Placeholder variant)
- Created src/gdb/target_desc.rs with TARGET_DESCRIPTION_XML = "" (empty string)
- Added pub mod gdb; to src/lib.rs
- cargo build passes successfully

### Pattern observed:
- Module pattern in src/lib.rs: `pub mod module_name;`
- Files go directly in src/gdb/ (not nested further)

---

## Task 2 Completion Notes (2026-04-13)

### What was done:
- Added `use std::collections::HashSet;` import to src/processor/mod.rs
- Added `debug_mode: bool` and `sw_breakpoints: HashSet<u64>` fields to ProcessorState
- Initialized both fields in ProcessorState::new() (debug_mode: false, sw_breakpoints: HashSet::new())
- Added accessor methods: debug_mode(), set_debug_mode(), sw_breakpoints(), add_sw_breakpoint(), remove_sw_breakpoint()
- Defined DebugStopReason enum with Breakpoint(u64), StepComplete, Halted variants
- Modified one_step() to return Option<DebugStopReason> with debug interception logic
- Modified step() to return Option<DebugStopReason> and break early on debug stop
- Modified step_with_debug() to return Result<Option<DebugStopReason>, String>
- Fixed caller in src/bin/terminus.rs (added semicolon after .unwrap())
- Added 5 unit tests, all passing

### Key patterns:
- Bus is a trait; TerminusBus is the concrete type used in tests
- Processor::new takes (hartid, config, &Rc<B>, clint, plic) where B: Bus
- When debug_mode=false, one_step() returns None (equivalent to previous unit return)
- sw_breakpoints check uses next_pc (the address about to be executed) before execute_one()
- EBREAK interception checks Trap::Exception(Exception::Breakpoint) after execute_one() fails

### Files modified:
- src/processor/mod.rs (main changes)
- src/bin/terminus.rs (caller fix: semicolon after .unwrap())

---

## Task 4 Completion Notes (2026-04-13)

### What was done:
- Implemented `RiscvRegId` enum with variants: `Gpr(u16)`, `Pc`, `Fpr(u16)`, `Fcsr`, `Csr(u16)`, `Priv`
- Implemented `gdbstub::arch::RegId` trait for `RiscvRegId`
- `from_raw_id()` returns `Option<(Self, Option<NonZeroUsize>)>` per gdbstub 0.7 API
- Created CSR mapping table with 26 common CSRs (sstatus through mhartid)
- Created `TARGET_DESCRIPTION_XML` static string with 4 features: cpu, fpu, csr, virtual
- All 8 unit tests pass

### Key Finding - gdbstub RegId trait signature:
The `from_raw_id` method returns `Option<(Self, Option<NonZeroUsize>)>` not `Option<Self>`.
The second value is the register size in bytes, or None for virtual registers.

### Register Numbering Summary:
- 0-31: GPR (x0-x31)
- 32: PC
- 33-64: FPR (f0-f31) 
- 65: FCSR (fflags, frm, fcsr combined at this ID per GDB spec)
- 66-91: CSRs (26 common CSRs)
- 132: priv (virtual, privilege level)

### CSR Mapping (66-91):
Index 0 -> sstatus (0x100)
Index 1 -> sie (0x104)
...
Index 25 -> mhartid (0xF14)

### Files:
- src/gdb/reg_id.rs - RiscvRegId enum + RegId trait + CSR helpers
- src/gdb/target_desc.rs - TARGET_DESCRIPTION_XML constant

---

## Task 6 Completion Notes (2026-04-13)

### What was done:
- Implemented `GdbTarget` struct in `src/gdb/target.rs`:
  - Uses `*mut Processor` (raw pointer) to avoid lifetime issues with `BlockingEventLoop` trait
  - Implements `Target` trait with `Arch = Riscv64`, `Error = String`
  - Implements `SingleThreadBase`: read/write registers (GPR + PC), read/write memory via LoadStore
  - Implements `SingleThreadResume`: `resume()` sets `ExecMode::Continue`
  - Implements `SingleThreadSingleStep`: `step()` sets `ExecMode::Step`
  - Implements `SwBreakpoint`: add/remove via `ProcessorState::add_sw_breakpoint/remove_sw_breakpoint`
  - `guard_rail_implicit_sw_breakpoints()` returns `true` (we handle breakpoints ourselves)
  - Memory access uses `LoadStore::load_byte/store_byte` through MMU translation

- Implemented `GdbEventLoop` in `src/gdb/event_loop.rs`:
  - Implements `BlockingEventLoop` trait with `Target = GdbTarget`, `Connection = TcpStream`
  - `wait_for_stop_reason()`:
    - Step mode: executes one instruction, returns DoneStep or SwBreak
    - Continue mode: executes in batches of 1024, checks for GDB data between batches
    - Maps `DebugStopReason::Breakpoint` → `SingleThreadStopReason::SwBreak(())`
    - Maps `DebugStopReason::StepComplete`/None → `SingleThreadStopReason::DoneStep`
    - Maps `DebugStopReason::Halted` → `SingleThreadStopReason::Terminated(Signal::SIGSTOP)`
  - `on_interrupt()` returns `SingleThreadStopReason::Signal(Signal::SIGINT)`
  - `wait_for_gdb_connection()`: TCP listener, accepts one connection

- Integrated with `src/bin/terminus.rs`:
  - Added `--gdb` CLI flag (optional, defaults to 0.0.0.0:1234)
  - When `--gdb` specified: enters GDB debug session via `run_gdb_session()`
  - When not specified: normal simulation loop runs unchanged (zero overhead)

### Key Findings:
- `SingleThreadStopReason::SwBreak` takes `()` (unit) for single-threaded targets, not the address
- `TcpStream` already implements `Connection` and `ConnectionExt` in gdbstub (with std feature)
- `BlockingEventLoop` trait requires `type Target` without lifetime parameters, so `GdbTarget` uses `*mut Processor`
- `GdbTarget` needs `unsafe impl Send + Sync` because of the raw pointer
- `Processor::step(1)` returns `Option<DebugStopReason>` which maps directly to GDB stop reasons
- `Processor::one_step()` checks `sw_breakpoints` before execution and intercepts `Exception::Breakpoint` after
- The `--gdb` flag uses `is_present()` check with `value_of().unwrap_or("0.0.0.0:1234")` for default address

### Files modified:
- src/gdb/target.rs (GdbTarget implementation)
- src/gdb/event_loop.rs (GdbEventLoop + wait_for_gdb_connection)
- src/gdb/mod.rs (updated exports)
- src/bin/terminus.rs (added --gdb flag and GDB session integration)

---

## Task 7 Completion Notes (2026-04-13)

### What was done:
- Extracted simulation loop into reusable `run_simulation_loop()` function:
  - Takes all necessary parameters (sys, step, trace_file, virtio devices, etc.)
  - Called from main() when --gdb is NOT specified
  - Zero overhead when GDB is not used
  
- Enhanced `run_gdb_session()` function in src/bin/terminus.rs:
  - Creates TcpListener and binds to specified address
  - Loops to accept GDB connections
  - Handles disconnect/reconnect properly:
    - On disconnect: prints message and loops back to accept()
    - On reconnect: simulation resumes automatically
    - On TargetExited/TargetTerminated/Kill: exits loop
  - Uses TcpStream with read timeout for non-blocking checks

- Address parsing:
  - --gdb :1234 → listens on all interfaces, port 1234
  - --gdb 127.0.0.1:1234 → listens on specific interface
  - Default when flag present without value: 0.0.0.0:1234

### Key Patterns:
- `TcpListener::bind()` accepts both ":1234" and "127.0.0.1:1234" formats
- GDB session runs in a loop to support reconnect:
  ```rust
  loop {
      let (connection, _) = listener.accept()?;
      // Run GDB session
      // On disconnect, continue loop
  }
  ```
- Simulation is paused implicitly when waiting for `listener.accept()`
- `DisconnectReason::Disconnect` indicates clean client disconnect
- Connection errors also trigger disconnect handling

### Files Modified:
- src/bin/terminus.rs:
  - Added constants at module level (CORE_FREQ, TIMER_FREQ, etc.)
  - Extracted `run_simulation_loop()` function
  - Updated `run_gdb_session()` with disconnect/reconnect handling
  - Restructured main() to call extracted functions

### Verification:
✅ cargo build passes
✅ cargo test passes (17 tests)
✅ --gdb flag appears in --help output
✅ --gdb :12345 starts GDB server
✅ Normal execution without --gdb unchanged (zero overhead)

---

## Task 8 Completion Notes (2026-04-13)

### What was done:
- Created `tests/gdb_integration.rs` with comprehensive integration tests
- Added 6 unit tests to `src/gdb/target_desc.rs` for XML validation
- Added 7 unit tests to `src/gdb/target.rs` for register/memory/breakpoint operations
- Existing tests in `src/gdb/reg_id.rs` already comprehensive (8 tests)
- All 31 unit tests pass
- Integration tests marked with `#[ignore]` when they require `riscv64-unknown-elf-gdb`
- cargo build passes
- cargo clippy shows no warnings in new code (warnings are in existing codebase)

### Files created:
- `tests/gdb_integration.rs` - Integration tests with GDB RSP protocol helpers

### Files modified:
- `src/gdb/target_desc.rs` - Added 6 unit tests for XML validation
- `src/gdb/target.rs` - Added 7 unit tests for target operations

### Test summary:
- Unit tests: 31 passed (including 15 new GDB module tests)
- Integration tests: 3 passed without GDB, 11 marked #[ignore] (require external GDB)

### Key learnings:
- GDB RSP (Remote Serial Protocol) packet format: `$data#checksum`
- Raw pointer handling in tests requires keeping the pointed-to value in scope
- Processor `pc()` returns current PC, `set_pc()` sets `next_pc` (executed on next fetch)
- `TargetError<String>` doesn't implement Debug, use `.is_ok()` instead of `.unwrap()`

