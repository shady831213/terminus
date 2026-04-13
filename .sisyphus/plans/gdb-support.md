# GDB Remote Debug Support for Terminus RISC-V Simulator

## TL;DR

> **Quick Summary**: Add GDB remote serial protocol support to the Terminus RISC-V simulator using the `gdbstub` crate, enabling `riscv64-unknown-elf-gdb` to connect via `target remote :1234` for step-by-step debugging, register inspection, memory access, and breakpoint control.
>
> **Deliverables**:
> - New `src/gdb/` module with GDB stub implementation
> - `GdbTarget` struct implementing gdbstub's `Target` trait
> - `GdbEventLoop` implementing `BlockingEventLoop` for simulation integration
> - Target description XML for RV64 + FPU + common CSRs
> - Debug-mode hooks in `Processor` for EBREAK interception and CSR bypass
> - CLI `--gdb :1234` flag for starting the GDB server
> - Unit and integration tests for GDB functionality
>
> **Estimated Effort**: Medium (8 tasks across 4 waves)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 → Task 5 → Task 7 → Task 8

---

## Context

### Original Request
Add GDB remote debug support to the Terminus RISC-V simulator so users can connect with `riscv64-unknown-elf-gdb` and debug running programs.

### Interview Summary
**Key Discussions**:
- **Implementation approach**: Use `gdbstub` crate (v0.7.x) rather than implementing RSP from scratch — battle-tested by OpenVMM, Firecracker, crosvm
- **Multi-hart**: Single-hart (hart 0) first, multi-hart vCont added later
- **CSRs**: Common CSRs only (mstatus, mtvec, mepc, mcause, sstatus, stvec, sepc, scause, satp, privilege)
- **XLen**: RV64 first, RV32 later
- **Memory access**: Physical addresses only (bypass MMU/PMP, standard debug behavior)
- **EBREAK**: Intercept ALL EBREAK instructions when GDB is attached
- **Startup**: Simulation waits for GDB connection before starting
- **Disconnect**: Simulation pauses when GDB disconnects
- **CLI**: `--gdb :1234` flag on existing terminus binary
- **Threading**: Integrated event loop (not separate thread)
- **Testing**: Tests after implementation using Rust `#[test]`

**Research Findings**:
- `gdbstub` crate provides `Target` trait, `Arch` trait, `SingleThreadBase` + resume ops, breakpoint ops, packet parsing, no-ack mode, built-in `TcpStream` `Connection` impl
- `gdbstub_arch::riscv::Riscv64` provides `RiscvCoreRegs<u64>` with `x: [u64; 32]` and `pc: u64` — needs custom `RegId` for FPU and CSR registers
- RISC-V register numbering: x0-x31 (0-31), pc (32), f0-f31 (33-64), CSRs (65+)
- Metis identified critical integration points: `set_pc()` sets `next_pc` not `pc`, `x0` is hardwired to 0, `FRegT = u128` needs conversion, CSR access needs privilege bypass, memory writes need TLB/icache flush

### Metis Review
**Identified Gaps** (all addressed):
- **EBREAK interception**: Must intercept ALL EBREAK before normal trap handler; add debug-mode flag to Processor
- **PC handling**: `set_pc()` sets `next_pc`; must also update `state.pc` for GDB visibility
- **x0 register**: Must explicitly zero `regs.x[0]` after reading since `xreg(0)` returns `&0`
- **FPU conversion**: `u128` internal → `u64`/`u32` for GDB; must handle NaN boxing per RISC-V spec
- **CSR privilege bypass**: Need `csr_debug()` / `set_csr_debug()` methods that skip privilege checks
- **Memory coherency**: Must flush TLB and icache after GDB memory writes
- **Breakpoint strategy**: PC-checking (HashSet of addresses), not instruction replacement
- **GDB memory access**: Physical addresses via Bus trait, bypassing MMU/PMP

---

## Work Objectives

### Core Objective
Implement GDB remote serial protocol support so that `riscv64-unknown-elf-gdb` can connect to the Terminus simulator, inspect/modify registers and memory, set breakpoints, step through code, and control execution.

### Concrete Deliverables
- `src/gdb/mod.rs` — Module root, public API
- `src/gdb/target.rs` — `GdbTarget` implementing gdbstub `Target` trait
- `src/gdb/event_loop.rs` — `GdbEventLoop` implementing `BlockingEventLoop`
- `src/gdb/reg_id.rs` — Custom `RegId` enum for FPU and CSR registers
- `src/gdb/target_desc.rs` — Target description XML for RV64 + FPU + CSRs
- `src/processor/mod.rs` — Debug-mode flag, EBREAK interception, CSR bypass methods
- `src/bin/terminus.rs` — `--gdb` CLI flag integration
- `tests/gdb_integration.rs` — Integration tests for GDB functionality

### Definition of Done
- [ ] `riscv64-unknown-elf-gdb` can connect via `target remote :1234`
- [ ] GDB can read/write all 32 integer registers (x0-x31) and PC
- [ ] GDB can read/write FPU registers (f0-f31, fcsr) when F/D extension enabled
- [ ] GDB can read/write memory using physical addresses
- [ ] GDB can single-step (`stepi`) and continue (`continue`)
- [ ] GDB can set/remove software breakpoints (`break *addr`, `clear`)
- [ ] GDB can interrupt execution with Ctrl-C
- [ ] GDB can read common CSRs (mstatus, mtvec, mepc, mcause, etc.) and priv
- [ ] Simulation waits for GDB connection when `--gdb` is specified
- [ ] Simulation pauses when GDB disconnects
- [ ] No-ack mode works (`QStartNoAckMode`)
- [ ] Target description XML correctly describes register set
- [ ] Normal simulation behavior unchanged when `--gdb` is NOT specified

### Must Have
- GDB RSP server using gdbstub crate (v0.7.x)
- Register access: x0-x31, PC, FPU (f0-f31, fcsr), common CSRs, priv
- Memory access: physical addresses bypassing MMU/PMP
- Execution control: continue, single-step, stop (Ctrl-C)
- Software breakpoints via PC-checking (Z0/z0)
- Target description XML for register discovery
- EBREAK interception in debug mode (intercept ALL EBREAK when GDB attached)
- `--gdb :1234` CLI flag
- Wait-for-connection startup behavior
- Pause-on-disconnect behavior
- No-ack mode support

### Must NOT Have (Guardrails)
- NO hardware breakpoints (Z1/z1) — simulator has no hardware breakpoint resources
- NO watchpoints (Z2/z2, Z3/z3, Z4/z4)
- NO multi-hart vCont support — single-hart only for initial implementation
- NO RV32 support — RV64 only for initial implementation
- NO RISC-V Debug Mode CSRs (dcsr, dpc, dscratch) — this is a GDB stub, not a debug module
- NO reverse debugging (reverse-step, reverse-continue)
- NO vRun (run program), extended mode, or flash operations
- NO instruction replacement for breakpoints — use PC-checking only (HashSet)
- NO dynamic CSR XML — static common CSRs only
- NO FPU register access bypassing FS state check initially — read regardless of FS
- NO separate thread for GDB — integrated event loop only
- AI slop patterns to avoid: over-abstraction, excessive comments, generic names

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (no unit test framework; only integration test examples)
- **Automated tests**: YES (tests after implementation)
- **Framework**: Rust built-in `#[test]` + integration tests with `riscv64-unknown-elf-gdb`
- **If TDD**: N/A (tests-after approach)

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **GDB functionality**: Use Bash to start terminus with `--gdb`, connect with `riscv64-unknown-elf-gdb` in batch mode, verify register/memory/step/breakpoint operations
- **Code quality**: Use `cargo build` + `cargo test` + `cargo clippy`
- **Integration**: End-to-end test with GDB client against simulator

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately - foundation + scaffolding):
├── Task 1: Add gdbstub dependency + create module skeleton [quick]
├── Task 2: Add debug-mode hooks to Processor [deep]
├── Task 3: Implement CSR bypass methods [quick]
└── Task 4: Create RegId enum + target description XML [quick]

Wave 2 (After Wave 1 - core GDB target):
├── Task 5: Implement GdbTarget (register + memory access) [deep]
└── Task 6: Implement GdbEventLoop + resume/step/breakpoints [deep]

Wave 3 (After Wave 2 - CLI integration + testing):
├── Task 7: CLI integration --gdb flag + main loop restructuring [unspecified-high]
└── Task 8: Integration tests + end-to-end QA [unspecified-high]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 5 → Task 7 → Task 8 → F1-F4 → user okay
Parallel Speedup: ~50% faster than sequential
Max Concurrent: 4 (Wave 1)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1    | -         | 5, 6   | 1    |
| 2    | -         | 5      | 1    |
| 3    | -         | 5      | 1    |
| 4    | -         | 5      | 1    |
| 5    | 1, 2, 3, 4| 7      | 2    |
| 6    | 1, 2      | 7      | 2    |
| 7    | 5, 6      | 8      | 3    |
| 8    | 7          | F1-F4  | 3    |

### Agent Dispatch Summary

- **Wave 1**: 4 tasks — T1 `quick`, T2 `deep`, T3 `quick`, T4 `quick`
- **Wave 2**: 2 tasks — T5 `deep`, T6 `deep`
- **Wave 3**: 2 tasks — T7 `unspecified-high`, T8 `unspecified-high`
- **FINAL**: 4 tasks — F1 `oracle`, F2 `unspecified-high`, F3 `unspecified-high`, F4 `deep`

---

## TODOs

- [x] 1. Add gdbstub dependency + create module skeleton

  **What to do**:
  - Add `gdbstub = "0.7"` and `gdbstub_arch = "0.3"` to `Cargo.toml` dependencies
  - Create `src/gdb/mod.rs` as module root with public exports
  - Add `pub mod gdb;` to `src/lib.rs`
  - Create `src/gdb/target.rs` with empty `GdbTarget` struct (just struct definition, no trait impls yet)
  - Create `src/gdb/event_loop.rs` with empty `GdbEventLoop` struct
  - Create `src/gdb/reg_id.rs` with placeholder `RiscvRegId` enum
  - Create `src/gdb/target_desc.rs` with placeholder `TARGET_DESCRIPTION_XML` constant (empty string for now)
  - Verify `cargo build` compiles without errors

  **Must NOT do**:
  - Do NOT implement any trait methods yet — skeleton only
  - Do NOT modify any existing code outside Cargo.toml and src/lib.rs
  - Do NOT add unnecessary dependencies

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Tasks 5, 6
  - **Blocked By**: None

  **References**:
  - `src/lib.rs:1-18` — Module declaration pattern (how modules are exported)
  - `src/devices/mod.rs` — Pattern for `pub mod` re-exports and module organization
  - gdbstub crate: https://docs.rs/gdbstub/0.7 — `Target` trait, `Arch` trait, `Connection` trait
  - gdbstub_arch::riscv::Riscv64 — Built-in RISC-V 64-bit arch definition

  **Acceptance Criteria**:
  - [ ] `cargo build` succeeds with no errors
  - [ ] `src/gdb/mod.rs`, `target.rs`, `event_loop.rs`, `reg_id.rs`, `target_desc.rs` exist
  - [ ] `pub mod gdb;` appears in `src/lib.rs`

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Project builds with gdbstub dependency
    Tool: Bash
    Preconditions: Cargo.toml updated with gdbstub/gdbstub_arch deps
    Steps:
      1. Run `cargo build` in project root
      2. Check exit code is 0
      3. Run `grep -r "pub mod gdb" src/lib.rs` to verify module is exported
    Expected Result: Build succeeds, module is exported
    Failure Indicators: Build errors, missing module declaration
    Evidence: .sisyphus/evidence/task-1-build-success.txt
  ```

  **Commit**: YES (group with Tasks 2, 3, 4)
  - Message: `feat(gdb): add gdbstub dependency and module skeleton`
  - Files: `Cargo.toml`, `src/lib.rs`, `src/gdb/mod.rs`, `src/gdb/target.rs`, `src/gdb/event_loop.rs`, `src/gdb/reg_id.rs`, `src/gdb/target_desc.rs`

- [x] 2. Add debug-mode hooks to Processor

  **What to do**:
  - Add a `debug_mode: bool` field to `ProcessorState` (private, default `false`)
  - Add `debug_mode()` and `set_debug_mode(val: bool)` accessors to `ProcessorState`
  - Add `sw_breakpoints: HashSet<u64>` field to `ProcessorState` for breakpoint addresses (need `use std::collections::HashSet`)
  - Add `sw_breakpoints()`, `add_sw_breakpoint(addr: u64)`, `remove_sw_breakpoint(addr: u64)` methods
  - Add a `DebugStopReason` enum: `Breakpoint(u64)`, `StepComplete`, `Halted`
  - Modify `one_step()` / `step_with_debug()` in `Processor` to check `debug_mode` flag:
    - Before `handle_trap()` for `Exception::Breakpoint`: if `debug_mode` is true, skip normal trap handling and return `DebugStopReason::Breakpoint(pc)` instead
    - After instruction fetch: if PC is in `sw_breakpoints` and `debug_mode` is true, return `DebugStopReason::Breakpoint(pc)` BEFORE executing the instruction
  - Ensure existing behavior is 100% unchanged when `debug_mode` is `false`

  **Must NOT do**:
  - Do NOT change behavior when `debug_mode` is `false`
  - Do NOT implement hardware breakpoints (Z1)
  - Do NOT add Debug Mode CSRs (dcsr/dpc/dscratch)
  - Do NOT modify the `Exception` enum
  - Do NOT add watchpoint support

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: [`rust-skills:m01-ownership`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3, 4)
  - **Blocks**: Tasks 5, 6
  - **Blocked By**: None

  **References**:
  - `src/processor/mod.rs:471-481` — `step()` execution loop: where debug stop checks go
  - `src/processor/mod.rs:483-508` — `step_with_debug()`: must coexist
  - `src/processor/mod.rs:55-68` — `ProcessorState` struct: where new fields go
  - `src/processor/trap.rs:32` — `Exception::Breakpoint`: the exception to intercept

  **Acceptance Criteria**:
  - [ ] `debug_mode` field exists on `ProcessorState` with getter/setter
  - [ ] `sw_breakpoints: HashSet<u64>` field exists on `ProcessorState`
  - [ ] `DebugStopReason` enum is defined
  - [ ] When `debug_mode=false`, existing behavior is unchanged
  - [ ] When `debug_mode=true`, EBREAK returns `DebugStopReason::Breakpoint` instead of entering trap handler

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Debug mode flag does not affect normal execution
    Tool: Bash
    Preconditions: debug_mode field added, default false
    Steps:
      1. Run `cargo build` to verify compilation
      2. Run `cargo test` or existing integration tests
    Expected Result: Build succeeds, existing tests pass, no regressions
    Failure Indicators: Build errors, test failures
    Evidence: .sisyphus/evidence/task-2-no-regression.txt

  Scenario: EBREAK interception works in debug mode
    Tool: Bash (cargo test)
    Preconditions: debug_mode = true, Processor encounters EBREAK
    Steps:
      1. Write a unit test that creates Processor with debug_mode = true
      2. Step through an EBREAK instruction
      3. Verify DebugStopReason::Breakpoint is returned (NOT trap entry)
    Expected Result: EBREAK returns debug stop reason, trap handler is NOT invoked
    Failure Indicators: EBREAK causes trap entry to M-mode, or no debug stop detected
    Evidence: .sisyphus/evidence/task-2-ebreak-intercept.txt
  ```

  **Commit**: YES (group with Tasks 1, 3, 4)
  - Message: `feat(processor): add debug-mode hooks and EBREAK interception`
  - Files: `src/processor/mod.rs`

- [x] 3. Implement CSR bypass methods

  **What to do**:
  - Add `csr_debug(&self, addr: u64) -> u64` method to `ProcessorState` that reads CSR values without privilege checking
  - Add `set_csr_debug(&mut self, addr: u64, val: u64)` method that writes CSR values without privilege checking
  - These methods access the same CSR storage as `csr()`/`set_csr()` but skip `csr_privilege_check()`
  - Map common CSRs: mstatus(0x300), misa(0x301), medeleg(0x302), mideleg(0x303), mie(0x304), mtvec(0x305), mscratch(0x340), mepc(0x341), mcause(0x342), mtval(0x343), mip(0x344), mcycle(0xB00), minstret(0xB02), mvendorid(0xF11), marchid(0xF12), mimpid(0xF13), mhartid(0xF14), sstatus(0x100), sie(0x104), stvec(0x105), sscratch(0x140), sepc(0x141), scause(0x142), stval(0x143), sip(0x144), satp(0x180)
  - Add `privilege_to_u8(&self) -> u8` method that returns current privilege level (U=0, S=1, M=3)

  **Must NOT do**:
  - Do NOT modify `csr()` or `set_csr()` — they must continue to enforce privilege checks
  - Do NOT implement Debug Mode CSRs (dcsr/dpc/dscratch)
  - Do NOT add CSR addresses beyond the common set

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 4)
  - **Blocks**: Task 5
  - **Blocked By**: None

  **References**:
  - `src/processor/mod.rs:217-235` — `ProcessorState::csr()` and `set_csr()`: pattern to follow
  - `src/processor/privilege/m/csrs.rs:1-396` — M-mode CSR definitions
  - `src/processor/privilege/s/csrs.rs` — S-mode CSR definitions

  **Acceptance Criteria**:
  - [ ] `csr_debug()` reads CSRs regardless of privilege mode
  - [ ] `set_csr_debug()` writes CSRs regardless of privilege mode
  - [ ] `privilege_to_u8()` returns correct values (U=0, S=1, M=3)
  - [ ] Original `csr()`/`set_csr()` still enforce privilege checks

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: CSR debug bypasses privilege
    Tool: Bash (cargo test)
    Steps:
      1. Unit test: create ProcessorState in U-mode
      2. Call csr_debug(0x300) — mstatus, normally inaccessible from U-mode
      3. Verify method returns mstatus value without exception
    Expected Result: csr_debug reads M-mode CSRs regardless of privilege
    Failure Indicators: csr_debug returns error, panic, or wrong value
    Evidence: .sisyphus/evidence/task-3-csr-debug-bypass.txt
  ```

  **Commit**: YES (group with Tasks 1, 2, 4)
  - Message: `feat(processor): add CSR debug-access and privilege bypass`
  - Files: `src/processor/mod.rs`, `src/processor/privilege/mod.rs`

- [x] 4. Create RegId enum + target description XML

  **What to do**:
  - Create `RiscvRegId` enum in `src/gdb/reg_id.rs` with variants: `Gpr(u16)`, `Pc`, `Fpr(u16)`, `Fcsr`, `Csr(u16)`, `Priv`
  - Implement gdbstub's `RegId` trait for `RiscvRegId`:
    - `fn from_raw_id(id: usize) -> Option<Self>` — GDB numbering: 0-31=GPR, 32=PC, 33-64=FPR, 65+=CSR
  - Create CSR mapping table: GDB register number → CSR address for common CSRs
  - Create target description XML in `src/gdb/target_desc.rs`:
    - Feature `org.gnu.gdb.riscv.cpu`: x0-x31 (bitsize=64, type=int/code_ptr) + pc (regnum 0-32)
    - Feature `org.gnu.gdb.riscv.fpu`: f0-f31 (bitsize=64, type=ieee_double) + fflags + frm + fcsr
    - Feature `org.gnu.gdb.riscv.csr`: common CSRs (regnum 65+)
    - Feature `org.gnu.gdb.riscv.virtual`: priv (bitsize=8, type=int)

  **Must NOT do**:
  - Do NOT include Debug Mode CSRs
  - Do NOT generate dynamic XML — static const string only
  - Do NOT include vector registers
  - Do NOT implement RV32 size variants yet

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 3)
  - **Blocks**: Task 5
  - **Blocked By**: None

  **References**:
  - `src/processor/extensions/f/mod.rs:70-75` — `ExtensionF` struct: FPU register storage
  - `src/global.rs:1-55` — Type definitions (`InsnT`, `RegT`, `FRegT`, `XLen`)
  - gdbstub `RegId` trait: https://docs.rs/gdbstub/0.7/gdbstub/arch/trait.RegId.html
  - GDB RISC-V features: https://sourceware.org/gdb/current/onlinedocs/gdb.html/RISC_002dV-Features.html

  **Acceptance Criteria**:
  - [ ] `RiscvRegId::from_raw_id()` maps all register numbers correctly (0-32 for GPR+PC, 33-64 for FPR, 65+ for CSR)
  - [ ] Target description XML contains all 4 features (cpu, fpu, csr, virtual)
  - [ ] XML is valid and parseable by GDB

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: RegId mapping correctness
    Tool: Bash (cargo test)
    Steps:
      1. Unit test: from_raw_id(0) → Gpr(0), from_raw_id(32) → Pc, from_raw_id(33) → Fpr(0), from_raw_id(65) → first CSR
      2. from_raw_id(9999) → None
    Expected Result: All register number mappings return correct variants
    Failure Indicators: Wrong variant, panic, None for valid registers
    Evidence: .sisyphus/evidence/task-4-regid-mapping.txt
  ```

  **Commit**: YES (group with Tasks 1, 2, 3)
  - Message: `feat(gdb): add gdbstub dependency and module skeleton`
  - Files: `src/gdb/reg_id.rs`, `src/gdb/target_desc.rs`

- [ ] 5. Implement GdbTarget (register + memory access)

  **What to do**:
  - Implement `gdbstub::target::Target` trait for `GdbTarget`:
    - `type Arch = gdbstub_arch::riscv::Riscv64`
    - `type Error = std::io::Error`
    - `fn base_ops(&mut self) -> BaseOps<Self::Arch, Self::Error>` — delegate to SingleThreadBase impl
  - Implement `SingleThreadBase` for `GdbTarget`:
    - `read_registers(&mut self, regs: &mut RiscvCoreRegs<u64>)` — fill regs from ProcessorState: `x[0..32]` from `xreg()`, `pc` from `pc()`. **CRITICAL**: Set `regs.x[0] = 0` after reading since `xreg(0)` returns `&0`.
    - `write_registers(&mut self, regs: &RiscvCoreRegs<u64>)` — write regs to ProcessorState: `set_xreg()` for each register, `set_pc()` for PC. **CRITICAL**: After `set_pc()`, also update `state.pc` directly for immediate visibility.
  - Implement `SingleRegisterAccess` for `GdbTarget`:
    - `read_register(&mut self, regid: RiscvRegId, val: &mut Vec<u8>)` — handle GPR, PC, FPR, FCSR, CSR, Priv reads via RiscvRegId dispatch
    - `write_register(&mut self, regid: RiscvRegId, val: &[u8])` — handle writes to same register types
    - CSR reads use `csr_debug()` (bypasses privilege), FPR reads extract lower 64 bits from `FRegT` (u128), Priv reads use `privilege_to_u8()`
  - Implement `SingleThreadResume` for `GdbTarget`:
    - `resume(&mut self)` — non-blocking, sets execution mode (Continue/Step)
  - Implement `SwBreakpoint` for `GdbTarget`:
    - `add_sw_breakpoint(&mut self, addr: u64)` — add to `sw_breakpoints` HashSet
    - `remove_sw_breakpoint(&mut self, addr: u64)` — remove from `sw_breakpoints` HashSet
  - Implement memory access in `GdbTarget`:
    - `read_addrs(&mut self, start_addr: u64, data: &mut [u8])` — physical memory via Bus trait
    - `write_addrs(&mut self, start_addr: u64, data: &[u8])` — write via Bus then flush TLB/icache

  **Must NOT do**:
  - Do NOT use `ProcessorState::csr()` for GDB reads — must use `csr_debug()`
  - Do NOT forget to zero `regs.x[0]` after reading registers
  - Do NOT forget to set both `next_pc` and `pc` when GDB writes PC
  - Do NOT implement hardware breakpoints (Z1/z1)
  - Do NOT implement watchpoints (Z2-Z4)
  - Do NOT modify instruction memory for breakpoints — use HashSet PC-checking
  - Do NOT forget to flush TLB/icache after GDB memory writes

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: [`rust-skills:m01-ownership`, `rust-skills:m04-zero-cost`]

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 1-4)
  - **Parallel Group**: Wave 2 (with Task 6)
  - **Blocks**: Task 7
  - **Blocked By**: Tasks 1, 2, 3, 4

  **References**:
  - `src/processor/mod.rs:471-508` — `step()` and `step_with_debug()`: execution loop
  - `src/processor/mod.rs:294-298` — `pc()` and `set_pc()`: CRITICAL — set_pc sets next_pc
  - `src/processor/mod.rs:318-328` — `xreg()` and `set_xreg()`: x0 hardwired to 0
  - `src/processor/mod.rs:217-235` — `csr()` and `set_csr()`: pattern for csr_debug()
  - `src/devices/bus.rs:1-193` — `Bus` trait: physical memory access
  - `src/processor/mmu/mod.rs:53` — `Mmu`: flush after writes
  - gdbstub `Target`: https://docs.rs/gdbstub/0.7/gdbstub/target/trait.Target.html
  - gdbstub `SingleThreadBase`: https://docs.rs/gdbstub/0.7/gdbstub/target/ext/base/singlethread/trait.SingleThreadBase.html
  - gdbstub `SwBreakpoint`: https://docs.rs/gdbstub/0.7/gdbstub/target/ext/breakpoints/trait.SwBreakpoint.html

  **Acceptance Criteria**:
  - [ ] All gdbstub trait methods implemented for GdbTarget
  - [ ] Register read/write round-trip: read → write → read returns consistent values
  - [ ] x0 always reads as 0 even after writing
  - [ ] PC write updates both pc and next_pc
  - [ ] CSR access bypasses privilege checking
  - [ ] Memory read/write uses physical addresses (Bus trait)
  - [ ] TLB flushed after memory writes
  - [ ] Software breakpoints managed via HashSet

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Register read/write round-trip
    Tool: Bash (cargo test)
    Steps:
      1. Unit test: create ProcessorState with known values
      2. Call read_registers() → verify x0=0, x1-x31 match, pc matches
      3. Call write_registers() with new values → verify set_xreg called
      4. Call read_registers() again → verify values updated, x0 still 0
      5. Write PC → verify both pc and next_pc updated
    Expected Result: All register ops consistent, x0 always 0
    Failure Indicators: x0 not zero, PC not updated, wrong values
    Evidence: .sisyphus/evidence/task-5-register-roundtrip.txt

  Scenario: Memory read/write with TLB flush
    Tool: Bash (cargo test)
    Steps:
      1. Write known bytes to physical address via write_addrs()
      2. Read back same address via read_addrs()
      3. Verify written bytes match read bytes
    Expected Result: Memory round-trip succeeds
    Failure Indicators: Wrong bytes, TLB not flushed
    Evidence: .sisyphus/evidence/task-5-memory-access.txt

  Scenario: Software breakpoint management
    Tool: Bash (cargo test)
    Steps:
      1. add_sw_breakpoint(0x80000000) → verify in HashSet
      2. add_sw_breakpoint(0x80000100) → verify both present
      3. remove_sw_breakpoint(0x80000000) → verify only 0x80000100 remains
    Expected Result: Breakpoints added and removed correctly
    Failure Indicators: HashSet not updated
    Evidence: .sisyphus/evidence/task-5-sw-breakpoints.txt
  ```

  **Commit**: YES (separate commit)
  - Message: `feat(gdb): implement GdbTarget with register, memory, and breakpoint access`
  - Files: `src/gdb/target.rs`

- [ ] 6. Implement GdbEventLoop + resume/step/breakpoints

  **What to do**:
  - Implement `SingleThreadSingleStep` for `GdbTarget`:
    - `step(&mut self)` — execute one instruction then stop
  - Create `GdbEventLoop` struct implementing `gdbstub::stub::BlockingEventLoop`:
    - `wait_for_stop_reason()` — main event loop:
      - If mode is Continue: execute instructions in batches (1024 steps), check for GDB data between batches
      - If mode is Step: execute exactly one instruction, return `StopReason::DoneWithStep`
      - After each step: check if PC is in `sw_breakpoints` → return `StopReason::SwBreak`
      - If `Exception::Breakpoint` intercepted (debug_mode) → return `StopReason::SwBreak`
      - On Ctrl-C (interrupt) → return `StopReason::Interrupted`
    - Handle GDB packet processing between instruction batches
    - On disconnect: pause simulation, wait for reconnect
  - Create `GdbConnection` for TCP:
    - Listen on specified address (`0.0.0.0:1234`)
    - Accept one connection at a time
    - On disconnect: pause, wait for new connection
  - Integrate with `GdbStub::run_blocking()`
  - Ensure zero overhead when `--gdb` is NOT specified

  **Must NOT do**:
  - Do NOT run GDB in a separate thread
  - Do NOT modify instructions in memory for breakpoints
  - Do NOT implement hardware breakpoints or watchpoints
  - Do NOT forget to check for Ctrl-C between instruction batches
  - Do NOT add overhead when `--gdb` is not specified

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: [`rust-skills:m07-concurrency`, `rust-skills:m01-ownership`]

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 1, 2)
  - **Parallel Group**: Wave 2 (with Task 5)
  - **Blocks**: Task 7
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `src/bin/terminus.rs:133-466` — Main simulation loop: must restructure
  - gdbstub `BlockingEventLoop`: https://docs.rs/gdbstub/0.7/gdbstub/stub/trait.BlockingEventLoop.html
  - gdbstub `GdbStub::run_blocking()`: https://docs.rs/gdbstub/0.7/gdbstub/stub/struct.GdbStub.html
  - gdbstub armv4t example: https://github.com/daniel5151/gdbstub/blob/master/examples/armv4t/src/emulator.rs

  **Acceptance Criteria**:
  - [ ] Simulation runs correctly without `--gdb` flag (no regression)
  - [ ] Simulation waits for GDB connection when `--gdb` specified
  - [ ] `stepi` advances PC by one instruction
  - [ ] `continue` resumes execution
  - [ ] Ctrl-C stops execution
  - [ ] GDB disconnect pauses simulation, reconnect works

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: No regression without --gdb
    Tool: Bash
    Steps:
      1. Run `cargo build`
      2. Run `terminus examples/linux/image/br-5-4` (without --gdb)
      3. Verify simulation runs identically to pre-GDB version
    Expected Result: Normal execution, no GDB overhead
    Failure Indicators: Different behavior, crashes, slower execution
    Evidence: .sisyphus/evidence/task-6-no-gdb-regression.txt

  Scenario: Continue and step via GDB
    Tool: Bash
    Steps:
      1. Start `terminus --gdb :12345 <elf>`
      2. Connect with `riscv64-unknown-elf-gdb -batch -ex "target remote :12345" -ex "stepi" -ex "print/x \$pc" <elf>`
      3. Verify PC advances by one instruction
      4. Send `continue` then Ctrl-C and verify execution stops
    Expected Result: stepi advances PC, continue runs, Ctrl-C stops
    Failure Indicators: stepi doesn't advance, continue hangs, Ctrl-C doesn't stop
    Evidence: .sisyphus/evidence/task-6-continue-step.txt
  ```

  **Commit**: YES (separate commit)
  - Message: `feat(gdb): implement GdbEventLoop with resume, step, and breakpoints`
  - Files: `src/gdb/event_loop.rs`, `src/gdb/target.rs`

- [ ] 7. CLI integration --gdb flag + main loop restructuring

  **What to do**:
  - Add `--gdb <addr>` CLI flag to `src/bin/terminus.rs` using existing `clap` v2 argument parser
  - Format: `--gdb :1234` (listen on all interfaces, port 1234) or `--gdb 127.0.0.1:1234` (specific interface)
  - When specified: create GdbTarget, TcpListener, start GdbEventLoop instead of normal simulation loop
  - When NOT specified: run existing simulation loop unchanged (zero overhead)
  - Restructure `main()`: extract simulation loop into a function callable from both normal and GDB paths
  - When `--gdb`: create GdbTarget from Processor/System, create TCP listener, run `GdbStub::run_blocking()`
  - Update `src/gdb/mod.rs` to expose: `GdbTarget`, `GdbEventLoop`, `run_gdb_server()`
  - Handle GDB disconnect/reconnect: pause simulation on disconnect, resume on reconnect

  **Must NOT do**:
  - Do NOT change simulation behavior when `--gdb` is NOT specified
  - Do NOT add overhead to normal simulation path
  - Do NOT remove or change existing CLI flags
  - Do NOT implement multiple simultaneous GDB connections

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (with Task 8)
  - **Blocks**: Task 8
  - **Blocked By**: Tasks 5, 6

  **References**:
  - `src/bin/terminus.rs:1-466` — Main binary with CLI parsing and simulation loop
  - `src/bin/terminus.rs:133-151` — Network option parsing: pattern for CLI flags
  - `src/system/mod.rs:1-533` — `System` struct with `processors()` method

  **Acceptance Criteria**:
  - [ ] `--gdb :1234` flag starts GDB server and waits for connection
  - [ ] Normal execution without `--gdb` is unchanged
  - [ ] GDB connect, detach, reconnect works
  - [ ] Simulation pauses when GDB disconnects

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: --gdb flag starts GDB server
    Tool: Bash
    Steps:
      1. Run `terminus --gdb :12345 <elf> &`
      2. Run `ss -tlnp | grep 12345` to verify listening
      3. Verify process stays running after connection close
    Expected Result: Listens on port, accepts connections
    Failure Indicators: Port not listening, process exits
    Evidence: .sisyphus/evidence/task-7-gdb-flag.txt

  Scenario: No regression without --gdb
    Tool: Bash
    Steps:
      1. Run `terminus <elf>` (without --gdb)
      2. Verify output identical to pre-GDB version
    Expected Result: Identical behavior
    Failure Indicators: Different output, crashes, slower
    Evidence: .sisyphus/evidence/task-7-no-gdb-regression.txt
  ```

  **Commit**: YES (separate commit)
  - Message: `feat(gdb): integrate GDB server with CLI and main simulation loop`
  - Files: `src/bin/terminus.rs`, `src/gdb/mod.rs`, `src/gdb/event_loop.rs`

- [ ] 8. Integration tests + end-to-end QA

  **What to do**:
  - Create `tests/gdb_integration.rs` with integration tests (connection, register read/write, memory read/write, step, continue, breakpoints, Ctrl-C, no-ack, CSR, privilege, target description, detach, FPU)
  - Create a small test ELF binary or use existing one
  - Add test helper for starting terminus with `--gdb` in background
  - Add unit tests in `src/gdb/` modules for RegId mapping, XML validity, register/memory access
  - Mark GDB integration tests `#[ignore]` (require `riscv64-unknown-elf-gdb`)
  - Run `cargo test` and `cargo clippy`

  **Must NOT do**:
  - Do NOT require `riscv64-unknown-elf-gdb` for default `cargo test`
  - Do NOT modify existing tests
  - Do NOT add flaky timing-dependent tests
  - Do NOT test multi-hart features

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (sequential after Task 7)
  - **Blocks**: Final Verification Wave
  - **Blocked By**: Task 7

  **References**:
  - `top_tests/riscv_tests.rs:1-479` — Existing integration test pattern
  - `top_tests/htif_test.rs` — Another integration test example

  **Acceptance Criteria**:
  - [ ] `cargo test --lib` passes
  - [ ] `cargo clippy -- -D warnings` clean
  - [ ] GDB connection and register read test passes
  - [ ] GDB breakpoint and step test passes
  - [ ] GDB memory read/write test passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: All unit tests pass
    Tool: Bash
    Steps:
      1. Run `cargo test --lib`
      2. Verify all unit tests pass
      3. Run `cargo clippy -- -D warnings`
    Expected Result: All tests pass, clippy clean
    Failure Indicators: Any test failure, any clippy warning
    Evidence: .sisyphus/evidence/task-8-unit-tests.txt

  Scenario: GDB end-to-end connection
    Tool: Bash
    Steps:
      1. Start `terminus --gdb :12346 <test-elf> &`
      2. `riscv64-unknown-elf-gdb -batch -ex "target remote :12346" -ex "info registers" <test-elf>`
      3. Verify register values shown, x0=0
    Expected Result: GDB connects, registers readable
    Failure Indicators: Connection refused, register read fails
    Evidence: .sisyphus/evidence/task-8-gdb-register-read.txt

  Scenario: GDB breakpoint and step
    Tool: Bash
    Steps:
      1. Start terminus with --gdb
      2. GDB: `break *0x80000000`, `continue`, verify PC at breakpoint, `stepi`, verify PC advances
    Expected Result: Breakpoint stops execution, stepi advances PC
    Failure Indicators: Breakpoint doesn't stop, stepi doesn't advance
    Evidence: .sisyphus/evidence/task-8-gdb-breakpoint-step.txt
  ```

  **Commit**: YES (separate commit)
  - Message: `test(gdb): add integration and unit tests for GDB remote debug`
  - Files: `tests/gdb_integration.rs`, `src/gdb/reg_id.rs`, `src/gdb/target_desc.rs`, `src/gdb/target.rs`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
>
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, curl endpoint, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo build` + `cargo clippy` + `cargo test`. Review all changed files for: `as any`/`unsafe` without SAFETY comment, empty catches, `println!` in non-test code, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names (data/result/item/temp).
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (features working together, not isolation). Test edge cases: empty state, invalid input, rapid actions. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT Have" compliance. Detect cross-task contamination: Task N touching Task M's files. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **Commit 1**: `feat(gdb): add gdbstub dependency and module skeleton` — Cargo.toml, src/gdb/mod.rs, src/lib.rs, src/processor/ changes
- **Commit 2**: `feat(processor): add debug-mode hooks and EBREAK interception` — src/processor/mod.rs, src/processor/privilege/
- **Commit 3**: `feat(gdb): implement GdbTarget with register, memory, and breakpoint access` — src/gdb/target.rs
- **Commit 4**: `feat(gdb): implement GdbEventLoop with resume, step, and breakpoints` — src/gdb/event_loop.rs, src/gdb/target.rs
- **Commit 5**: `feat(gdb): integrate GDB server with CLI and main simulation loop` — src/bin/terminus.rs, src/gdb/mod.rs
- **Commit 6**: `test(gdb): add integration and unit tests for GDB remote debug` — tests/

---

## Success Criteria

### Verification Commands
```bash
cargo build                                                       # Expected: Build succeeds
cargo test                                                        # Expected: All tests pass
cargo clippy -- -D warnings                                      # Expected: No warnings
terminus --gdb :1234 <elf-file> &                                 # Expected: Waits for GDB connection
riscv64-unknown-elf-gdb -batch -ex "target remote :1234" <elf>   # Expected: Connects successfully
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
- [ ] GDB connection works
- [ ] Register read/write works
- [ ] Memory read/write works
- [ ] Continue and step work
- [ ] Breakpoints work
- [ ] Ctrl-C interrupt works
- [ ] No-ack mode works
- [ ] CSR access works
- [ ] Simulation waits for connection on --gdb
- [ ] Simulation pauses on GDB disconnect