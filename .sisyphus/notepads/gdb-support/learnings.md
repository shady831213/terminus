# Task 4 Progress Notes

## Date: 2026-04-13

## Completed
- Created `src/gdb/reg_id.rs` with `RiscvRegId` enum implementing `gdbstub::arch::RegId`
- Created `src/gdb/target_desc.rs` with `TARGET_DESCRIPTION_XML` constant

## GDB Register Numbering
- 0-31: GPR (x0-x31) - 8 bytes each
- 32: PC - 8 bytes
- 33-64: FPR (f0-f31) - 8 bytes each
- 65: FCSR - 8 bytes
- 66-91: CSRs (mstatus, misa, ..., satp) - 8 bytes each
- 132: priv (virtual) - 1 byte

## CSR Mapping
- CSR address 0x100-0x13F maps to GDB registers 66-91
- `csr_addr_to_regnum()` helper converts CSR addr to GDB regnum

## Blocker
- Build fails due to Task 2's incomplete changes to `ProcessorState`
- Task 2 added `debug_mode: bool` and `sw_breakpoints: HashSet<u64>` fields
- Constructor in `new()` method not updated to initialize these fields
- Awaiting Task 2 completion to unblock build

---

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
