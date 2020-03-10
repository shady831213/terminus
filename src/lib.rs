extern crate terminus_macros;
extern crate linkme;
extern crate terminus_proc_macros;

mod insn;
mod execption;
mod decode;

use linkme::*;

#[cfg(test)]
mod test;