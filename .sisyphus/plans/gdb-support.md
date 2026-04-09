# Plan: Add GDB Support to terminus RISC-V Simulator

**Goal**: Integrate GDB Remote Serial Protocol (RSP) support into the terminus RISC-V simulator so users can debug programs running in the simulator using GDB via `--gdb-port <PORT>`.

**Library**: `gdbstub = "0.7"` + `gdbstub_arch = "0.3"` (provides `Riscv64` arch with `RiscvCoreRegs<u64>`)

**Scope**: RV64 only, single-hart (hart 0), TCP connection, software breakpoints only.

---

## TODOs

- [x] **T1**: Add `gdbstub = "0.7"` and `gdbstub_arch = "0.3"` to `[dependencies]` in `Cargo.toml`. Run `cargo check` to confirm it compiles.

- [x] **T2**: Add physical memory access helpers to `Processor` in `src/processor/mod.rs`:
  - `pub fn read_mem_physical(&self, paddr: u64, buf: &mut [u8]) -> Result<(), String>` — reads bytes from the bus at physical address, byte by byte
  - `pub fn write_mem_physical(&self, paddr: u64, data: &[u8]) -> Result<(), String>` — writes bytes to the bus at physical address, byte by byte
  - `pub fn translate_vaddr(&self, vaddr: u64) -> Result<u64, String>` — translates a virtual address to physical using `mmu.ls_translate()`; returns `Err` on page fault (do NOT fall back to identity mapping — let the caller decide)

- [x] **T3**: Add `step_gdb(n: usize, gdb_breakpoints: &std::collections::HashSet<u64>) -> StepResult` to `Processor` in `src/processor/mod.rs` (the set contains **guest virtual addresses** of installed GDB breakpoints):
  - Define `pub enum StepResult { Ok, Breakpoint, Exit(String) }` (in same file or `src/processor/debug.rs`)
  - `step_gdb` loops up to `n` times calling the same WFI-gate + `execute_one()` logic as `one_step()`:
    - If `execute_one()` returns `Err(Trap::Exception(Exception::Breakpoint))`:
      - **Check if `state.pc` (guest virtual address) is in `gdb_breakpoints`**. If YES → return `StepResult::Breakpoint` WITHOUT calling `handle_trap()`. If NO → the guest intentionally executed `ebreak` → call `handle_trap(trap)` as normal and continue.
      - `state.pc` is the guest virtual PC (set to `next_pc` at start of `one_insn`). The `gdb_breakpoints` set must also use guest virtual addresses so the comparison is in the same address space.
      - This distinction is critical: without it, all guest `ebreak` instructions are swallowed by GDB and the guest's breakpoint exception handler never runs.
    - If `execute_one()` returns any other `Err(trap)`: call `handle_trap(trap)` as normal, continue loop
    - If `EXIT_CTRL.poll()` returns `Ok(msg)`: return `StepResult::Exit(msg)`
  - After the loop, call `ext.step_cb(self)` for all extensions (same as `step()`)
  - The `gdb_breakpoints` set contains **guest virtual addresses** of installed GDB breakpoints. The `TerminusTarget` breakpoint map should be keyed by virtual address (the `addr` parameter GDB sends in Z0/z0 packets) — NOT physical address. Physical address is only needed internally for the actual memory patch.

- [x] **T4**: Create `src/gdb/mod.rs` implementing the `gdbstub` `Target` trait for terminus:
  - `TerminusTarget<'a>` struct holds `sys: &'a mut System`, a `HashMap<u64, Vec<u8>>` for saved breakpoint bytes, and a `SimContext` struct with device handles needed by the event loop (virtio_console, virtio_net, step_cnt, single_step flag)
  - Implement `Target` for `TerminusTarget<'a>`:
    - `type Arch = gdbstub_arch::riscv::Riscv64`
    - `type Error = String`
    - `base_ops()` → `BaseOps::SingleThread(self)`
    - `support_breakpoints()` → `Some(self)`
  - Implement `SingleThreadBase` (all 4 required methods + `support_resume() -> Some(self)`):
    - `read_registers`: fill `RiscvCoreRegs<u64>` from hart 0's `xreg(i)` and **`next_pc()`** (NOT `pc()` — `next_pc` is the architectural PC GDB should see: the address of the next instruction to execute; `pc()` is an internal field set only during `one_insn()` execution)
    - `write_registers`: call `set_xreg(i, v)` for i=1..31, `set_pc(v)` for pc (sets `next_pc`)
    - `read_addrs(start_addr: u64, data: &mut [u8]) -> TargetResult<usize, Self>`: per-byte VA→PA translation + physical read; return number of bytes successfully read; return `TargetError::NonFatal` on translation/bus error
    - `write_addrs(start_addr: u64, data: &[u8]) -> TargetResult<(), Self>`: per-byte VA→PA + physical write
  - Implement `SingleThreadResume`: `resume()` sets `ctx.single_step = false`; `support_single_step()` returns `Some(self)`
  - Implement `SingleThreadSingleStep`: `step()` sets `ctx.single_step = true`
  - Implement `Breakpoints` + `SwBreakpoint`:
    - `Breakpoints::support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>>` → **must return `Some(self)`** (default is `None`; without this override, Z0/z0 packets are silently ignored)
    - `add_sw_breakpoint(addr, kind)`: `addr` is the guest virtual address GDB sends. Read and write breakpoint bytes **one byte at a time**, translating each byte's VA→PA individually using `mmu.fetch_translate(state, &(addr + i), 1)`. This correctly handles 32-bit instructions that straddle a page boundary (e.g., at page offset `0xffe` — the simulator's own fetcher already handles this case in `src/processor/fetcher.rs`). Save `kind` original bytes keyed by **virtual address `addr`**. Write ebreak bytes byte-by-byte. Flush icache.
    - `remove_sw_breakpoint(addr, kind)`: same per-byte fetch_translate approach, restore saved bytes (looked up by virtual `addr`), flush icache
  - Add `pub mod gdb;` to `src/lib.rs`

- [x] **T5**: Integrate GDB into `src/bin/terminus.rs`:
  - Add `--gdb-port PORT` CLI argument (optional u16)
  - Validate: if `--gdb-port` is set, require `core_num == 1` and `xlen == X64`; exit with error otherwise
  - After `sys.reset(...)`, if `--gdb-port` is set: bind TCP listener, print waiting message, accept one connection, then run `GdbStub::new(stream).run_blocking::<GdbEventLoop>(&mut target)` where `GdbEventLoop` implements `BlockingEventLoop`:
    - `wait_for_stop_reason(target, conn)`:
      - Poll `conn` for incoming GDB data: call `ConnectionExt::peek()` which returns `Result<Option<u8>, _>`. If `Some(_)` (data available), then call `conn.read()` to **consume** the byte, and return `Event::IncomingData(byte)`. Do NOT return the peeked byte directly — `peek()` does not consume it from the socket buffer.
      - Run `step_gdb(1, &breakpoint_vaddr_set)` if `ctx.single_step`, else `step_gdb(STEP, &breakpoint_vaddr_set)` where STEP=500; `breakpoint_vaddr_set` is the set of **guest virtual addresses** from `target.breakpoints.keys()` (the map is keyed by VA)
      - On `StepResult::Breakpoint` → return `Event::TargetStopped(SingleThreadStopReason::SwBreak(()))`
      - On `StepResult::Exit(msg)` → print msg, return `Event::TargetStopped(SingleThreadStopReason::Exited(0))`
      - On `StepResult::Ok` with `single_step=true` → return `Event::TargetStopped(SingleThreadStopReason::DoneStep)` immediately. Do NOT try to distinguish "instruction retired" vs "WFI/interrupt/fault" — `step_gdb(1)` represents one step attempt; GDB expects to stop after each `stepi` regardless of whether the PC advanced (trap delivery, WFI, etc. are all valid stop points for a debugger).
      - Drive timer/console/net polling same as normal loop (using `ctx.step_cnt` and `ctx.virtio_console`, `ctx.virtio_net`)
    - `on_interrupt(target)` → return `Some(SingleThreadStopReason::Signal(Signal::SIGINT))`
  - After `run_blocking` returns, call `term_exit()` and return (skip normal loop)
  - When `--gdb-port` not set: existing loop unchanged

- [x] **T6**: Document GDB usage in `README.md`:
  - Add a `## GDB Debugging` section explaining `--gdb-port`, single-core/RV64 requirement, and example GDB session (`target remote`, `info registers`, `stepi`, `break *addr`, `continue`)

---

## Final Verification Wave

- [x] **F1 - Build**: `cargo build` and `cargo build --release` succeed with zero errors/warnings
- [x] **F2 - No Regressions**: `cargo test` and existing riscv_tests pass unchanged (no `--gdb-port` = identical behavior)
- [x] **F3 - GDB Works**: Smoke test: simulator starts with `--gdb-port 1234` and prints "Waiting for GDB connection on port 1234..."; multi-core rejection works (`-p 2` exits with error). Full GDB session test requires `riscv64-unknown-elf-gdb` (not installed in this environment).
- [x] **F4 - Code Quality**: No bare `unwrap()` in GDB paths (`src/gdb/mod.rs` has zero unwrap calls — uses `ok_or(TargetError::NonFatal)?` and `map_err`), no `unsafe` code added.

---

## Key Technical Notes for Implementor

### gdbstub 0.7 exact API signatures (verified from docs.rs)
```rust
// SingleThreadBase
fn read_registers(&mut self, regs: &mut RiscvCoreRegs<u64>) -> TargetResult<(), Self>
fn write_registers(&mut self, regs: &RiscvCoreRegs<u64>) -> TargetResult<(), Self>
fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<usize, Self>
fn write_addrs(&mut self, start_addr: u64, data: &[u8]) -> TargetResult<(), Self>
fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>>  // return Some(self)

// SingleThreadResume
fn resume(&mut self, signal: Option<Signal>) -> Result<(), Self::Error>
fn support_single_step(&mut self) -> Option<SingleThreadSingleStepOps<'_, Self>>

// SingleThreadSingleStep
fn step(&mut self, signal: Option<Signal>) -> Result<(), Self::Error>

// SwBreakpoint
fn add_sw_breakpoint(&mut self, addr: u64, kind: usize) -> TargetResult<bool, Self>
fn remove_sw_breakpoint(&mut self, addr: u64, kind: usize) -> TargetResult<bool, Self>
```

### System accessor for hart 0
Use `sys.processor(0).unwrap()` (returns `Option<&mut Processor>`) — NOT `processors()[0]` or `processors_mut()` (the latter doesn't exist).

### PC semantics (CRITICAL — read carefully)
- `state.next_pc()` = **the architectural PC GDB should expose** — address of next instruction to execute
- `state.pc()` = internal field, only valid DURING `one_insn()` execution (set to `next_pc` at start of `one_insn`, then `next_pc` is updated by the instruction)
- After `reset()`: `pc = 0`, `next_pc = start_address` → GDB must show `next_pc` (start_address), not `pc` (0)
- After EBREAK: `pc = address_of_ebreak`, `next_pc = address_of_ebreak + instruction_size` → GDB should show `pc` (the ebreak address) so it can identify the breakpoint
- **Rule**: `read_registers` uses `next_pc()` in normal halted state; after EBREAK interception in `step_gdb`, `pc` already equals the ebreak address (set by `one_insn` before execution), so `next_pc` at that point equals ebreak_addr + size. Use `pc()` after breakpoint hit, `next_pc()` otherwise.
- **Simplest correct approach**: always expose `next_pc()` — GDB will show the address of the next instruction to execute, which is correct for `stepi` and initial attach. After EBREAK, `next_pc` points past the ebreak; GDB will adjust PC back to the breakpoint address itself using the `swbreak` stop reason.
- `write_registers` calls `set_pc(regs.pc)` → sets `next_pc` → takes effect on next step ✓

### Software breakpoint bytes
- kind=4 (32-bit EBREAK): `[0x73, 0x00, 0x10, 0x00]`
- kind=2 (16-bit C.EBREAK): `[0x02, 0x90]`

### ConnectionExt::peek() in gdbstub 0.7
Returns `Result<Option<u8>, Connection::Error>` — `Some(byte)` if data available, `None` if not. Use this instead of raw `TcpStream::set_nonblocking`.

### EXIT_CTRL polling
`terminus_spaceport::EXIT_CTRL.poll()` returns `Ok(msg)` when exit is requested. Must be checked in `step_gdb()` loop to support clean shutdown in GDB mode.
