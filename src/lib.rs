pub extern crate terminus_macros;
extern crate linkme;
pub extern crate terminus_proc_macros;
pub extern crate terminus_global;

use linkme::*;

mod insn;
mod execption;
mod decode;
mod processor;
mod extentions;
