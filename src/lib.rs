extern crate terminus_macros;
pub extern crate linkme;
pub extern crate terminus_proc_macros;
pub mod insn;
pub use linkme::*;
#[cfg(test)]
mod test;