# GDB Support Implementation Decisions

## Decisions Log

### [2026-04-13] Starting Wave 1 Implementation
- Decision: Run Tasks 1-4 in parallel as they have no dependencies
- Rationale: Maximum parallelization for foundation tasks

### Dependencies to Add
- gdbstub = "0.7"
- gdbstub_arch = "0.3"

### Module Structure
```
src/gdb/
├── mod.rs           # Module root, public API
├── target.rs        # GdbTarget implementing Target trait
├── event_loop.rs    # GdbEventLoop implementing BlockingEventLoop
├── reg_id.rs        # RiscvRegId enum
└── target_desc.rs   # Target description XML
```

### Integration Points
- `src/lib.rs`: Add `pub mod gdb;`
- `src/processor/mod.rs`: Add debug_mode flag, CSR bypass methods
- `src/bin/terminus.rs`: Add `--gdb` CLI flag
