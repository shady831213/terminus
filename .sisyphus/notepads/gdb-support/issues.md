
## [2026-04-09] Corrected plan v2 verification findings

- Plan is still not technically complete.
- `BlockingEventLoop` sample uses `type Target = TerminusTarget<'static>` while `TerminusTarget` holds `&mut System`; that lifetime is not valid. Event loop/target lifetimes need to be generic, or the target must own the required state differently.
- `SingleThreadBase::support_resume()` is still missing from the actionable T4 steps. In gdbstub 0.7, continue/step support is reached through `support_resume()`.
- `SingleThreadBase::read_addrs` in gdbstub 0.7 takes `start_addr: Arch::Usize` and returns `TargetResult<usize, Self>`. The plan text still describes the older/wrong signature shape.
- `wait_for_stop_reason` does not return `SingleThreadStopReason::DoneStep` after a successful single-step, so `stepi` would keep running instead of stopping after one instruction.
- `Processor::step_gdb()` as described bypasses the existing `wfi` handling in `one_step()`, which is a behavioral regression.
- `read_registers`/`write_registers` need to respect Terminus's split `pc`/`next_pc` state. Reading `pc` and writing only `next_pc` will misreport the architectural PC after normal stepping and after GDB register writes.
- The proposed `wait_for_stop_reason` body references local device handles (`virtio_console_device`, net, SDL state), but `BlockingEventLoop` callbacks only receive `target` and `conn`. Those handles must be stored on the target/context explicitly.
- Single-hart/RV64-only policy should be enforced at runtime when `--gdb-port` is used (`-p 1`, `-l 64`), not only documented.
- For socket polling, prefer `gdbstub::conn::ConnectionExt::peek()`. Do not rely on globally setting the `TcpStream` non-blocking unless verified against gdbstub's own connection behavior.

## [2026-04-09] Environment/tooling

- `lsp_diagnostics` could not be used for Rust verification in this environment because `rust-analyzer` is not installed in the active toolchain (`Unknown binary 'rust-analyzer'`). Relied on `cargo check` for compile validation instead.
