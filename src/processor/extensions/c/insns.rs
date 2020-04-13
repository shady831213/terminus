// use terminus_global::*;
// use terminus_macros::*;
// use terminus_proc_macros::Instruction;
// use crate::processor::{Processor, Privilege, PrivilegeLevel};
// use crate::processor::trap::Exception;
// use crate::processor::insn::*;
// use crate::processor::decode::*;
// use crate::linkme::*;
//
// #[derive(Instruction)]
// #[format(CL)]
// #[code("0b????????????????010???????????10")]
// #[derive(Debug)]
// struct CLWSP(InsnT);
//
// impl Execution for CLWSP {
//     fn execute(&self, p: &Processor) -> Result<(), Exception> {
//     }
// }