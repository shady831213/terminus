use terminus_global::*;
use terminus_macros::*;
use terminus_proc_macros::Instruction;
use crate::processor::Processor;
use crate::processor::execption::Exception;
use crate::processor::insn::*;
use crate::processor::decode::*;
use crate::linkme::*;
use std::num::Wrapping;


trait Branch: InstructionImp {
    fn branch<F: Fn(RegT, RegT) -> bool>(&self, p: &Processor, condition: F) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        if condition(rs1, rs2) {
            p.state().set_pc((offset + pc).0);
        } else {
            p.state().set_pc(pc.0 + 4);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????000?????1100011")]
#[derive(Debug)]
struct BEQ(InsnT);

impl Branch for BEQ {}

impl Execution for BEQ {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.branch(p, |rs1, rs2| { rs1 == rs2 })
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????001?????1100011")]
#[derive(Debug)]
struct BNE(InsnT);

impl Branch for BNE {}

impl Execution for BNE {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.branch(p, |rs1, rs2| { rs1 != rs2 })
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????100?????1100011")]
#[derive(Debug)]
struct BLT(InsnT);

impl Branch for BLT {}

impl Execution for BLT {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.branch(p, |rs1, rs2| { (sext(rs1, p.state.config().xlen.len()) as SRegT) < (sext(rs2, p.state.config().xlen.len()) as SRegT) })
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????101?????1100011")]
#[derive(Debug)]
struct BGE(InsnT);

impl Branch for BGE {}

impl Execution for BGE {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.branch(p, |rs1, rs2| { (sext(rs1, p.state.config().xlen.len()) as SRegT) > (sext(rs2, p.state.config().xlen.len()) as SRegT) })
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????110?????1100011")]
#[derive(Debug)]
struct BLTU(InsnT);

impl Branch for BLTU {}

impl Execution for BLTU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.branch(p, |rs1, rs2| { rs1 < rs2 })
    }
}

#[derive(Instruction)]
#[format(B)]
#[code("0b?????????????????111?????1100011")]
#[derive(Debug)]
struct BGEU(InsnT);

impl Branch for BGEU {}

impl Execution for BGEU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.branch(p, |rs1, rs2| { rs1 > rs2 })
    }
}


trait Jump: InstructionImp {
    fn jump<F: Fn(Wrapping<RegT>) -> Wrapping<RegT>>(&self, p: &Processor, target: F) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        p.state().set_pc(target(offset).0);
        p.state().set_xreg(self.rd() as RegT, p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????1100111")]
#[derive(Debug)]
struct JALR(InsnT);

impl Jump for JALR {}

impl Execution for JALR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.jump(p, |offset| { offset + Wrapping(p.state().xreg(self.rs1() as RegT)) })
    }
}

#[derive(Instruction)]
#[format(J)]
#[code("0b?????????????????????????1101111")]
#[derive(Debug)]
struct JAL(InsnT);

impl Jump for JAL {}

impl Execution for JAL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.jump(p, |offset| { offset + Wrapping(p.state().pc()) })
    }
}


trait CsrAccess: InstructionImp {
    fn csr_access<F: Fn(RegT) -> RegT>(&self, p: &Processor, csr_value: F, valid: bool) -> Result<(), Exception> {
        if valid {
            let csr = p.state().csr(self.imm() as RegT)?;
            p.state().set_csr(self.imm() as RegT, csr_value(csr))?;
            p.state().set_pc(p.state().pc() + 4);
            p.state().set_xreg(self.rd() as RegT, csr);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????001?????1110011")]
#[derive(Debug)]
struct CSRRW(InsnT);

impl CsrAccess for CSRRW {}

impl Execution for CSRRW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, | _| { p.state().xreg(self.rs1() as RegT) }, self.rd() != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????1110011")]
#[derive(Debug)]
struct CSRRS(InsnT);
impl CsrAccess for CSRRS {}
impl Execution for CSRRS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |csr| { p.state().xreg(self.rs1() as RegT) | csr }, self.rs1() != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????1110011")]
#[derive(Debug)]
struct CSRRC(InsnT);
impl CsrAccess for CSRRC {}

impl Execution for CSRRC {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |csr| { !p.state().xreg(self.rs1() as RegT) & csr }, self.rs1() != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????101?????1110011")]
#[derive(Debug)]
struct CSRRWI(InsnT);
impl CsrAccess for CSRRWI {}

impl Execution for CSRRWI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |_| { self.rs1() as RegT }, self.rs1() != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????110?????1110011")]
#[derive(Debug)]
struct CSRRSI(InsnT);
impl CsrAccess for CSRRSI {}

impl Execution for CSRRSI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |csr| { self.rs1() as RegT | csr }, self.rs1() != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????111?????1110011")]
#[derive(Debug)]
struct CSRRCI(InsnT);
impl CsrAccess for CSRRCI {}

impl Execution for CSRRCI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |csr| { !self.rs1() as RegT & csr }, self.rs1() != 0)
    }
}