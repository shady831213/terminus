use crate::processor::insn_define::*;

// #[derive(Instruction)]
// #[format(CI)]
// #[code("0b????????????????010???????????10")]
// #[derive(Debug)]
// struct CLWSP(InsnT);
//
// impl Execution for CLWSP {
//     fn execute(&self, p: &Processor) -> Result<(), Exception> {
//         p.state().check_extension('c')?;
//         if self.rd() == 0 {
//             return Err(Exception::IllegalInsn(self.ir()));
//         }
//         let base: Wrapping<RegT> = Wrapping(p.state().xreg(2));
//         let offset_7_6: RegT = self.imm().bit_range(1, 0);
//         let offset_5: RegT = self.imm().bit_range(5, 5);
//         let offset_4_2: RegT = self.imm().bit_range(4, 2);
//         let offset: Wrapping<RegT> = Wrapping(offset_4_2 << 2 | offset_5 << 5 | offset_7_6 << 6);
//         let data = p.load_store().load_word((base + offset).0, p.mmu())?;
//         p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
//         p.state().set_pc(p.state().pc() + 2);
//         Ok(())
//     }
// }