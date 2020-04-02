use terminus_global::*;
use terminus_macros::*;
use terminus_proc_macros::Instruction;
use crate::processor::Processor;
use crate::processor::execption::Exception;
use crate::processor::insn::*;
use crate::processor::decode::*;
use crate::linkme::*;
use std::num::Wrapping;

#[derive(Instruction)]
#[format(J)]
#[code("0b?????????????????????????1101111")]
#[derive(Debug)]
struct JAL(InsnT);

impl Execution for JAL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().set_xreg(self.rd() as RegT, p.state().pc() + 4);
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, 20));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        p.state().set_pc((offset + pc).0);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????001?????1110011")]
#[derive(Debug)]
struct CSRRW(InsnT);

impl Execution for CSRRW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csr = p.state().csr(self.imm() as RegT)?;
        let rs = p.state().xreg(self.rs1() as RegT);
        p.state().set_csr(self.imm() as RegT, rs)?;
        p.state().set_pc(p.state().pc() + 4);
        p.state().set_xreg(self.rd() as RegT, csr);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????1110011")]
#[derive(Debug)]
struct CSRRS(InsnT);

impl Execution for CSRRS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csr = p.state().csr(self.imm() as RegT)?;
        let rs = p.state().xreg(self.rs1() as RegT);
        p.state().set_csr(self.imm() as RegT, rs | csr)?;
        p.state().set_pc(p.state().pc() + 4);
        p.state().set_xreg(self.rd() as RegT, csr);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????1110011")]
#[derive(Debug)]
struct CSRRC(InsnT);

impl Execution for CSRRC {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csr = p.state().csr(self.imm() as RegT)?;
        let rs = p.state().xreg(self.rs1() as RegT);
        p.state().set_csr(self.imm() as RegT, !rs & csr)?;
        p.state().set_pc(p.state().pc() + 4);
        p.state().set_xreg(self.rd() as RegT, csr);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????101?????1110011")]
#[derive(Debug)]
struct CSRRWI(InsnT);

impl Execution for CSRRWI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csr = p.state().csr(self.imm() as RegT)?;
        p.state().set_csr(self.imm() as RegT, self.rs1() as RegT)?;
        p.state().set_pc(p.state().pc() + 4);
        p.state().set_xreg(self.rd() as RegT, csr);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????110?????1110011")]
#[derive(Debug)]
struct CSRRSI(InsnT);

impl Execution for CSRRSI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csr = p.state().csr(self.imm() as RegT)?;
        p.state().set_csr(self.imm() as RegT, self.rs1() as RegT | csr)?;
        p.state().set_pc(p.state().pc() + 4);
        p.state().set_xreg(self.rd() as RegT, csr);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????111?????1110011")]
#[derive(Debug)]
struct CSRRCI(InsnT);

impl Execution for CSRRCI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csr = p.state().csr(self.imm() as RegT)?;
        p.state().set_csr(self.imm() as RegT, !(self.rs1() as RegT) & csr)?;
        p.state().set_pc(p.state().pc() + 4);
        p.state().set_xreg(self.rd() as RegT, csr);
        Ok(())
    }
}