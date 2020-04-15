use crate::processor::insn_define::*;
use std::num::Wrapping;
use crate::processor::extensions::f::FRegT;
use crate::processor::extensions::f::float::*;

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0000111")]
#[derive(Debug)]
struct FLD(InsnT);

impl FloatInsn for FLD {}

impl Execution for FLD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_double_word((base + offset).0, p.mmu())?;
        f.set_freg(self.rd() as RegT, data as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0100111")]
#[derive(Debug)]
struct FSD(InsnT);

impl FloatInsn for FSD {}

impl FStore for FSD {}

impl Execution for FSD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let data = f.freg(self.src());
        p.load_store().store_double_word((base + self.offset()).0, data as RegT, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}
