## [2026-04-09] Initial Research: Codebase + GDB RSP

### Codebase Architecture
- **Entry point**: `src/bin/terminus.rs` — CLI, system init, main loop
- **Main loop**: `for p in sys.processors() { p.step(500); }` — no pause mechanism
- **Processor**: `src/processor/mod.rs` — `ProcessorState` + `Processor`
- **ProcessorState fields**: `xreg: [RegT; 32]`, `pc: RegT`, `next_pc: RegT`, `ir: InsnT`, `wfi: bool`, `hartid: usize`
- **Register access**: `state.xreg(id)` / `state.set_xreg(id, val)` (public)
- **CSR access**: `state.csr(id)` / `state.set_csr(id, val)` (public, privilege-checked)
- **PC access**: `state.pc()` returns `&RegT`, `state.set_pc(val)` sets `next_pc`
- **Memory**: `LoadStore` struct with `load_byte/half_word/word/double_word` + store variants
- **Bus**: `TerminusBus` with `read_u8/u16/u32/u64` / `write_*` (physical address, no MMU)
- **Execution**: `one_step()` → `execute_one()` → `one_insn()` → fetch+decode+execute
- **WFI**: `state.wfi()` / `state.set_wfi(bool)` — pauses until interrupt

### Critical Constraint: Single-Threaded (Rc-based)
- `Processor`, `System`, `TerminusBus` all use `Rc<RefCell<...>>` — NOT `Send`/`Sync`
- Cannot move simulator to another thread
- Must use `GdbStubStateMachine` API (not `run_blocking`) to integrate GDB into main loop
- OR: use `run_blocking` on the SAME thread with non-blocking I/O polling

### GDB Integration Plan
- **Library**: `gdbstub = "0.7"` + `gdbstub_arch = "0.3"` 
- **Architecture type**: `gdbstub_arch::riscv::Riscv64` with `RiscvCoreRegs<u64>`
- **Register layout** (g packet): x0-x31 (8 bytes each) + pc (8 bytes) = 264 bytes total
- **Register numbering**: 0-31 GPRs, 32 PC, 33-64 FPRs, 65-4160 CSRs, 4161 priv
- **Connection**: TCP listener on configurable port (e.g., `--gdb-port 1234`)
- **Event loop strategy**: Integrate `GdbStubStateMachine` into the existing main loop

### GDB State Machine Integration Pattern
```
main loop:
  if gdb_active:
    poll GDB socket for incoming bytes
    if byte received: feed to GdbStubStateMachine
    if target stopped (breakpoint/step): report to GdbStubStateMachine
  else:
    p.step(N)  // normal execution
```

### Breakpoint Implementation
- Software breakpoints: replace instruction at addr with `ebreak` (0x00100073 for 32-bit, 0x9002 for 16-bit compressed)
- Store original bytes to restore on removal
- Check PC against breakpoint list in `one_step()` before executing

### Memory Access for GDB (physical address)
- GDB memory reads/writes use VIRTUAL addresses (as seen by the program)
- Need to translate VA→PA using MMU, then access bus
- Use `mmu.ls_translate()` or `mmu.fetch_translate()` for translation
- For physical-only access: use `bus.read_u8/write_u8` directly

### Key Files to Create/Modify
- `src/gdb/mod.rs` — new module: GDB stub implementation (Target trait)
- `src/gdb/target.rs` — Target trait impl for terminus
- `src/bin/terminus.rs` — add `--gdb-port` arg, integrate GDB state machine into loop
- `Cargo.toml` — add gdbstub + gdbstub_arch dependencies
- `src/lib.rs` — expose gdb module

## [2026-04-09] Oracle Review Corrections

### gdbstub 0.7 API — Correct Trait Structure
- `SingleThreadResume::resume(signal)` handles 'c' (continue) — NOT `ResumeAction` enum (that's old API)
- `SingleThreadSingleStep::step(signal)` handles 's' (stepi) — separate trait from Resume
- `support_single_step()` returns `Option<SingleThreadSingleStepOps<'_, Self>>` — must return `Some(self)` to enable
- `Target::base_ops()` must return `BaseOps::SingleThread(self)` — required method
- `Target::support_breakpoints()` returns `Option<BreakpointsOps<'_, Self>>` — opt-in

### EBREAK Interception — Critical Correctness Issue
- Current: `one_step()` calls `handle_trap()` on EBREAK → guest trap taken BEFORE GDB sees it
- Fix: `step_gdb()` intercepts `Err(Trap::Exception(Exception::Breakpoint))` BEFORE `handle_trap()`
- RISC-V: PC points TO the ebreak instruction (not past it) — `executed()` returns true so `pc = next_pc` was set
- Do NOT call `handle_trap()` for breakpoints in GDB mode

### Target::Arch is Compile-Time
- Cannot pick `Riscv32` vs `Riscv64` at runtime — it's an associated type
- Solution: RV64 only for initial implementation (covers the primary use case)
- RV32 support would require a separate binary or feature flag with conditional compilation

### Main Loop Preservation
- Raw `p.step(1)` in GDB mode would skip: timer ticks, console polling, net polling, SDL refresh
- Must replicate the `step_cnt >= CORE_STEP_TH` logic inside `wait_for_stop_reason`

### FPR/CSR in g packet
- `gdbstub_arch::riscv::Riscv64` uses `RiscvCoreRegs<u64>` = x0-x31 + pc ONLY
- FPRs (33-64) and CSRs (65-4160) are NOT in the default g packet
- Register number discussion in original plan was misleading — those numbers are for `p n` single-reg access
- For FPR/CSR support, need custom Arch impl or additional XML features (future work)

### Non-blocking TCP for GDB polling
- Use `stream.set_nonblocking(true)` + `stream.peek(&mut [0u8; 1])` to check for data
- Or use `ConnectionExt::peek()` from gdbstub's trait
- Must not block the simulation loop waiting for GDB data

## [2026-04-09] T2/T3 implementation notes

### Processor-side helpers
- `Processor::translate_vaddr` can reuse `mmu.ls_translate(state, &vaddr, 1, MmuOpt::Load)` and convert `Exception` into a user-facing `String`.
- Physical GDB memory access should bypass MMU and use the raw bus byte APIs so reads/writes work on arbitrary physical addresses.

### LoadStore access pattern
- `LoadStore::bus` is private to the submodule, so `Processor` needs a small accessor (`bus(&self) -> &Rc<dyn Bus>`) to implement physical memory helpers without exposing mutable internals.

### GDB stepping behavior
- `step_gdb` must mirror the existing `one_step` WFI gate and extension callback flow, but intercept `Trap::Exception(Exception::Breakpoint)` before `handle_trap`.
- GDB breakpoint detection compares the current virtual PC (`self.state().pc()`) against the external `HashSet<u64>`; non-GDB `ebreak` still flows through normal trap handling.
- EXIT handling belongs inside the GDB stepping loop via `terminus_spaceport::EXIT_CTRL.poll()` so GDB observes guest termination cleanly.

## [2026-04-09] Target module implementation details

- `VirtIOConsoleDevice` and `VirtIONetDevice` are available through `crate::devices::virtio_console` and `crate::devices::virtio_net` because `src/devices/mod.rs` re-exports `terminus_spaceport::devices::armory::*`.
- `SingleThreadBase::read_registers` should expose architectural GPRs plus `next_pc`, and `write_registers` should update `x1..x31` plus `set_pc`, matching Terminus's split `pc` / `next_pc` model.
- GDB memory packets should use `Processor::translate_vaddr(...).unwrap_or(va)` for normal load/store access, but software breakpoint patching must use `mmu().fetch_translate(state(), &va, 1)` so execute permission is enforced.
- Borrowing `self.sys.processor(0)` across `self.breakpoints` mutation is easiest to manage by scoping the processor borrow, then reacquiring it for icache-flushed patch writes/restores.

## [2026-04-09] terminus.rs GDB entrypoint integration

- `src/bin/terminus.rs` can keep the existing non-GDB `loop { ... }` untouched by branching immediately after `sys.reset(...)` and returning after the blocking GDB session ends.
- `SimContext` only needs cloned `Rc` handles to the console/net devices plus local `step_cnt` / `single_step` state; `Option<Rc<VirtIONetDevice>>` clones cleanly for the GDB path.
- `gdbstub 0.7` exposes both `peek()` and `read()` on `ConnectionExt`, so the blocking event loop can remain non-blocking on TCP without reaching into lower-level socket APIs.
- A lifetime-parameterized `GdbEventLoop<'a>` is the straightforward way to satisfy `BlockingEventLoop::Target = TerminusTarget<'a>` for a stack-borrowed `System`; forcing `TerminusTarget<'static>` is unnecessary.
- The blocking GDB loop should mirror the normal timer/console/net cadence with `STEP = 500`, `STEP_TH = 500`, and `TIMER_STEP = 50`, while mapping `StepResult::{Breakpoint, Exit, Ok}` to `SingleThreadStopReason` events.
