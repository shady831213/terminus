extern crate terminus_macros;
extern crate linkme;
extern crate terminus_proc_macros;
use linkme::*;

mod insn;
mod execption;
mod decode;


#[cfg(test)]
mod test;