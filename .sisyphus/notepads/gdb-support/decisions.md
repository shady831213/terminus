## [2026-04-09] Architecture Decisions

### Decision 1: Use GdbStubStateMachine (not run_blocking)
**Rationale**: terminus uses `Rc<RefCell<...>>` throughout — not `Send`. Cannot move simulator
to a separate thread. `GdbStubStateMachine` lets us drive the GDB protocol from the same
thread as the simulator, integrating into the existing main loop.

### Decision 2: TCP Connection on configurable port
**Rationale**: Standard approach for simulator GDB stubs. User passes `--gdb-port 1234`.
Simulator waits for GDB connection before starting execution (or starts and waits on first
`?` packet). `TcpStream` implements `gdbstub::Connection` automatically.

### Decision 3: Software breakpoints only (initially)
**Rationale**: Sufficient for basic debugging. Hardware breakpoints require RISC-V debug
module support which is not implemented. Software breakpoints: replace instruction with
`ebreak`, restore on removal.

### Decision 4: Virtual address memory access for GDB
**Rationale**: GDB sends virtual addresses. We need to translate using the MMU. Use
`mmu.ls_translate()` for data access. Fall back to physical if translation fails (e.g.,
when debugging M-mode code with no page tables).

### Decision 5: Support only hart 0 initially (single-thread GDB)
**Rationale**: Simplifies implementation. Multi-hart support can be added later using
`gdbstub`'s multi-thread extensions. Use `SingleThreadStopReason` and `SingleThreadBase`.

### Decision 6: gdbstub_arch version
**Rationale**: Use `gdbstub_arch = "0.3"` which pairs with `gdbstub = "0.7"`.
`RiscvCoreRegs<u64>` provides x0-x31 + pc. FP registers can be added later.
