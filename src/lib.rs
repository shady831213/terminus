extern crate terminus_macros;
extern crate linkme;
extern crate terminus_proc_macros;
use linkme::*;

mod insn;
mod execption;
mod decode;
mod processor;
mod extentions;
#[cfg(test)]
mod test;