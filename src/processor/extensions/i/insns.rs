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
#[format(B)]
#[code("0b?????????????????000?????1100011")]
#[derive(Debug)]
struct BEQ(InsnT);

impl Execution for BEQ {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        if rs1 == rs2 {
            p.state().set_pc((offset + pc).0);
        } else {
            p.state().set_pc(pc.0 + 4);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????001?????1100011")]
#[derive(Debug)]
struct BNE(InsnT);

impl Execution for BNE {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        if rs1 != rs2 {
            p.state().set_pc((offset + pc).0);
        } else {
            p.state().set_pc(pc.0 + 4);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????100?????1100011")]
#[derive(Debug)]
struct BLT(InsnT);

impl Execution for BLT {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        if (sext(rs1, p.state.config().xlen.len()) as SRegT) < (sext(rs2, p.state.config().xlen.len()) as SRegT) {
            p.state().set_pc((offset + pc).0);
        } else {
            p.state().set_pc(pc.0 + 4);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????101?????1100011")]
#[derive(Debug)]
struct BGE(InsnT);

impl Execution for BGE {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        if (sext(rs1, p.state.config().xlen.len()) as SRegT) > (sext(rs2, p.state.config().xlen.len()) as SRegT) {
            p.state().set_pc((offset + pc).0);
        } else {
            p.state().set_pc(pc.0 + 4);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????110?????1100011")]
#[derive(Debug)]
struct BLTU(InsnT);

impl Execution for BLTU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        if rs1 < rs2 {
            p.state().set_pc((offset + pc).0);
        } else {
            p.state().set_pc(pc.0 + 4);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????111?????1100011")]
#[derive(Debug)]
struct BGEU(InsnT);

impl Execution for BGEU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        if rs1 > rs2 {
            p.state().set_pc((offset + pc).0);
        } else {
            p.state().set_pc(pc.0 + 4);
        }
        Ok(())
    }
}


#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????1100111")]
#[derive(Debug)]
struct JALR(InsnT);

impl Execution for JALR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let rs1:Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        p.state().set_pc((offset + rs1).0);
        p.state().set_xreg(self.rd() as RegT, p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(J)]
#[code("0b?????????????????????????1101111")]
#[derive(Debug)]
struct JAL(InsnT);

impl Execution for JAL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        p.state().set_pc((offset + pc).0);
        p.state().set_xreg(self.rd() as RegT, p.state().pc() + 4);
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