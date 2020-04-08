use terminus_global::*;
use terminus_macros::*;
use terminus_proc_macros::Instruction;
use crate::processor::{Processor, Privilege};
use crate::processor::trap::Exception;
use crate::processor::insn::*;
use crate::processor::decode::*;
use crate::linkme::*;
use crate::processor::extensions::i::csrs::*;
use std::num::Wrapping;
use std::convert::TryFrom;


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
        self.branch(p, |rs1, rs2| { (sext(rs1, p.state.config().xlen.len()) as SRegT) >= (sext(rs2, p.state.config().xlen.len()) as SRegT) })
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
        self.branch(p, |rs1, rs2| { rs1 >= rs2 })
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


#[derive(Instruction)]
#[format(U)]
#[code("0b?????????????????????????0110111")]
#[derive(Debug)]
struct LUI(InsnT);

impl Execution for LUI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state.set_xreg(self.rd() as RegT, sext(self.imm() as RegT, self.imm_len()) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(U)]
#[code("0b?????????????????????????0010111")]
#[derive(Debug)]
struct AUIPC(InsnT);

impl Execution for AUIPC {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        p.state.set_xreg(self.rd() as RegT, (pc + offset).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0010011")]
#[derive(Debug)]
struct ADDI(InsnT);

impl Execution for ADDI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let rs2: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        p.state().set_xreg(self.rd() as RegT, (rs1 + rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0011011")]
#[derive(Debug)]
struct ADDIW(InsnT);

impl Execution for ADDIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs1() as RegT), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        p.state().set_xreg(self.rd() as RegT, sext((rs1 + rs2).0, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b000000???????????001?????0010011")]
#[derive(Debug)]
struct SLLI(InsnT);

impl Execution for SLLI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let shamt: RegT = (self.imm() as RegT).bit_range(p.state().config().xlen.len().trailing_zeros() as usize, 0);
        p.state().set_xreg(self.rd() as RegT, (rs1 << shamt) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0000000??????????001?????0011011")]
#[derive(Debug)]
struct SLLIW(InsnT);

impl Execution for SLLIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let shamt: RegT = (self.imm() as RegT).bit_range(4, 0);
        let low: RegT = (rs1 << shamt).bit_range(31, 0);
        p.state().set_xreg(self.rd() as RegT, low);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b000000???????????101?????0010011")]
#[derive(Debug)]
struct SRLI(InsnT);

impl Execution for SRLI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT) & p.state().config().xlen.mask();
        let shamt: RegT = (self.imm() as RegT).bit_range(p.state().config().xlen.len().trailing_zeros() as usize, 0);
        p.state().set_xreg(self.rd() as RegT, rs1 >> shamt);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0000000??????????101?????0011011")]
#[derive(Debug)]
struct SRLIW(InsnT);

impl Execution for SRLIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let shamt: RegT = (self.imm() as RegT).bit_range(4, 0);
        p.state().set_xreg(self.rd() as RegT, rs1 >> shamt);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b010000???????????101?????0010011")]
#[derive(Debug)]
struct SRAI(InsnT);

impl Execution for SRAI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT) & p.state().config().xlen.mask();
        let shamt: RegT = (self.imm() as RegT).bit_range(p.state().config().xlen.len().trailing_zeros() as usize, 0);
        let sign:RegT = sext(rs1.bit_range(p.state().config().xlen.len() -1, p.state().config().xlen.len() -1), 1);
        if shamt == 0 {
            p.state().set_xreg(self.rd() as RegT, rs1);
        } else {
            let shifted: RegT = sign.bit_range(shamt as usize - 1, 0);
            p.state().set_xreg(self.rd() as RegT, (rs1 >> shamt) | shifted << (p.state().config().xlen.len() as RegT - shamt));
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0100000??????????101?????0011011")]
#[derive(Debug)]
struct SRAIW(InsnT);

impl Execution for SRAIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let shamt: RegT = (self.imm() as RegT).bit_range(4, 0);
        let sign:RegT = sext(rs1.bit_range(31, 31), 1);
        if shamt == 0 {
            p.state().set_xreg(self.rd() as RegT, rs1);
        } else {
            let shifted: RegT = sign.bit_range(shamt as usize - 1, 0);
            p.state().set_xreg(self.rd() as RegT, (rs1 >> shamt) | shifted << (p.state().config().xlen.len() as RegT - shamt));
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0010011")]
#[derive(Debug)]
struct SLTI(InsnT);

impl Execution for SLTI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = sext(p.state().xreg(self.rs1() as RegT), p.state.config().xlen.len()) as SRegT;
        let rs2 = sext(self.imm() as RegT, self.imm_len()) as SRegT;
        if rs1 < rs2 {
            p.state().set_xreg(self.rd() as RegT, 1)
        } else {
            p.state().set_xreg(self.rd() as RegT, 0)
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0010011")]
#[derive(Debug)]
struct SLTIU(InsnT);

impl Execution for SLTIU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = sext(self.imm() as RegT, self.imm_len()) & p.state().config().xlen.mask();
        if self.rs1() == 0 || rs1 < rs2 {
            p.state().set_xreg(self.rd() as RegT, 1)
        } else {
            p.state().set_xreg(self.rd() as RegT, 0)
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????100?????0010011")]
#[derive(Debug)]
struct XORI(InsnT);

impl Execution for XORI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = sext(self.imm() as RegT, self.imm_len()) & p.state().config().xlen.mask();
        p.state().set_xreg(self.rd() as RegT, (rs1 ^ rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????110?????0010011")]
#[derive(Debug)]
struct ORI(InsnT);

impl Execution for ORI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = sext(self.imm() as RegT, self.imm_len()) & p.state().config().xlen.mask();
        p.state().set_xreg(self.rd() as RegT, (rs1 | rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????111?????0010011")]
#[derive(Debug)]
struct ANDI(InsnT);

impl Execution for ANDI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = sext(self.imm() as RegT, self.imm_len()) & p.state().config().xlen.mask();
        p.state().set_xreg(self.rd() as RegT, (rs1 & rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????000?????0110011")]
#[derive(Debug)]
struct ADD(InsnT);

impl Execution for ADD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let rs2: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs2() as RegT));
        p.state().set_xreg(self.rd() as RegT, (rs1 + rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????000?????0111011")]
#[derive(Debug)]
struct ADDW(InsnT);

impl Execution for ADDW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs1() as RegT), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs2() as RegT), 32));
        p.state().set_xreg(self.rd() as RegT, sext((rs1 + rs2).0, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0100000??????????000?????0110011")]
#[derive(Debug)]
struct SUB(InsnT);

impl Execution for SUB {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let rs2: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs2() as RegT));
        p.state().set_xreg(self.rd() as RegT, (rs1 - rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0100000??????????000?????0111011")]
#[derive(Debug)]
struct SUBW(InsnT);

impl Execution for SUBW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs1() as RegT), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs2() as RegT), 32));
        p.state().set_xreg(self.rd() as RegT, sext((rs1 - rs2).0, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????001?????0110011")]
#[derive(Debug)]
struct SLL(InsnT);

impl Execution for SLL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let shamt: RegT = p.state().xreg(self.rs2() as RegT).bit_range(p.state().config().xlen.len().trailing_zeros() as usize, 0);
        p.state().set_xreg(self.rd() as RegT, (rs1 << shamt) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0000000??????????001?????0111011")]
#[derive(Debug)]
struct SLLW(InsnT);

impl Execution for SLLW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let shamt: RegT = p.state().xreg(self.rs2() as RegT).bit_range(4, 0);
        let low: RegT = (rs1 << shamt).bit_range(31, 0);
        p.state().set_xreg(self.rd() as RegT, low);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????101?????0110011")]
#[derive(Debug)]
struct SRL(InsnT);

impl Execution for SRL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT) & p.state().config().xlen.mask();
        let shamt: RegT = p.state().xreg(self.rs2() as RegT).bit_range(p.state().config().xlen.len().trailing_zeros() as usize, 0);
        p.state().set_xreg(self.rd() as RegT, rs1 >> shamt);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0000000??????????101?????0111011")]
#[derive(Debug)]
struct SRLW(InsnT);

impl Execution for SRLW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let shamt: RegT = p.state().xreg(self.rs2() as RegT).bit_range(4, 0);
        p.state().set_xreg(self.rd() as RegT, rs1 >> shamt);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0100000??????????101?????0110011")]
#[derive(Debug)]
struct SRA(InsnT);

impl Execution for SRA {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT) & p.state().config().xlen.mask();
        let shamt: RegT = p.state().xreg(self.rs2() as RegT).bit_range(p.state().config().xlen.len().trailing_zeros() as usize, 0);
        let sign:RegT = sext(rs1.bit_range(p.state().config().xlen.len() -1, p.state().config().xlen.len() -1), 1);
        if shamt == 0 {
            p.state().set_xreg(self.rd() as RegT, rs1);
        } else {
            let shifted: RegT = sign.bit_range(shamt as usize - 1, 0);
            p.state().set_xreg(self.rd() as RegT, (rs1 >> shamt) | shifted << (p.state().config().xlen.len() as RegT - shamt));
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0100000??????????101?????0111011")]
#[derive(Debug)]
struct SRAW(InsnT);

impl Execution for SRAW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let shamt: RegT = p.state().xreg(self.rs2() as RegT).bit_range(4, 0);
        if shamt == 0 {
            p.state().set_xreg(self.rd() as RegT, rs1);
        } else {
            let shifted: RegT = rs1.bit_range(shamt as usize - 1, 0);
            p.state().set_xreg(self.rd() as RegT, (rs1 >> shamt) | shifted << (p.state().config().xlen.len() as RegT - shamt));
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????010?????0110011")]
#[derive(Debug)]
struct SLT(InsnT);

impl Execution for SLT {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = sext(p.state().xreg(self.rs1() as RegT), p.state.config().xlen.len()) as SRegT;
        let rs2 = sext(p.state().xreg(self.rs2() as RegT), p.state.config().xlen.len()) as SRegT;
        if rs1 < rs2 {
            p.state().set_xreg(self.rd() as RegT, 1)
        } else {
            p.state().set_xreg(self.rd() as RegT, 0)
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????011?????0110011")]
#[derive(Debug)]
struct SLTU(InsnT);

impl Execution for SLTU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        if self.rs2() != 0 && self.rs1() == 0 || rs1 < rs2 {
            p.state().set_xreg(self.rd() as RegT, 1)
        } else {
            p.state().set_xreg(self.rd() as RegT, 0)
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????100?????0110011")]
#[derive(Debug)]
struct XOR(InsnT);

impl Execution for XOR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        p.state().set_xreg(self.rd() as RegT, (rs1 ^ rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????110?????0110011")]
#[derive(Debug)]
struct OR(InsnT);

impl Execution for OR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        p.state().set_xreg(self.rd() as RegT, (rs1 | rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????111?????0110011")]
#[derive(Debug)]
struct AND(InsnT);

impl Execution for AND {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = p.state().xreg(self.rs2() as RegT);
        p.state().set_xreg(self.rd() as RegT, (rs1 & rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0000011")]
#[derive(Debug)]
struct LB(InsnT);

impl Execution for LB {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_byte((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 8) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????100?????0000011")]
#[derive(Debug)]
struct LBU(InsnT);

impl Execution for LBU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_byte((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????001?????0000011")]
#[derive(Debug)]
struct LH(InsnT);

impl Execution for LH {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_half_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 16) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????101?????0000011")]
#[derive(Debug)]
struct LHU(InsnT);

impl Execution for LHU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_half_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0000011")]
#[derive(Debug)]
struct LW(InsnT);

impl Execution for LW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????110?????0000011")]
#[derive(Debug)]
struct LWU(InsnT);

impl Execution for LWU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0000011")]
#[derive(Debug)]
struct LD(InsnT);

impl Execution for LD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_double_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait Store: InstructionImp {
    fn offset(&self) -> Wrapping<RegT> {
        let high: RegT = self.imm().bit_range(11, 5);
        let low = self.rd() as RegT;
        Wrapping(sext(high << 5 | low, self.imm_len()))
    }
    fn src(&self) -> RegT {
        self.imm().bit_range(4, 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0100011")]
#[derive(Debug)]
struct SB(InsnT);

impl Store for SB {}

impl Execution for SB {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let data = p.state().xreg(self.src());
        p.load_store.store_byte((base + self.offset()).0, data, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????001?????0100011")]
#[derive(Debug)]
struct SH(InsnT);

impl Store for SH {}

impl Execution for SH {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let data = p.state().xreg(self.src());
        p.load_store.store_half_word((base + self.offset()).0, data, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0100011")]
#[derive(Debug)]
struct SW(InsnT);

impl Store for SW {}

impl Execution for SW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let data = p.state().xreg(self.src());
        p.load_store.store_word((base + self.offset()).0, data, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0100011")]
#[derive(Debug)]
struct SD(InsnT);

impl Store for SD {}

impl Execution for SD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let data = p.state().xreg(self.src());
        p.load_store.store_double_word((base + self.offset()).0, data, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0001111")]
#[derive(Debug)]
struct FENCE(InsnT);

impl Execution for FENCE {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????001?????0001111")]
#[derive(Debug)]
struct FENCEI(InsnT);

impl Execution for FENCEI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        //fixme:no cache in fetcher, load_store and mmu for now
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait CsrAccess: InstructionImp {
    fn csr_access<F: Fn(RegT) -> RegT>(&self, p: &Processor, csr_value: F, read_csr: bool, write_csr: bool) -> Result<(), Exception> {
        let csr = if read_csr {
            p.state().csr(self.imm() as RegT)?
        } else {
            0
        };
        if write_csr {
            p.state().set_csr(self.imm() as RegT, csr_value(csr))?;
        }
        p.state().set_xreg(self.rd() as RegT, csr);
        p.state().set_pc(p.state().pc() + 4);
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
        self.csr_access(p, |_| { p.state().xreg(self.rs1() as RegT) }, self.rd() != 0, true)
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
        self.csr_access(p, |csr| { p.state().xreg(self.rs1() as RegT) | csr }, true, self.rs1() != 0)
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
        self.csr_access(p, |csr| { !p.state().xreg(self.rs1() as RegT) & csr }, true, self.rs1() != 0)
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
        self.csr_access(p, |_| { self.rs1() as RegT }, self.rd() != 0, true)
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
        self.csr_access(p, |csr| { self.rs1() as RegT | csr }, true, self.rs1() != 0)
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
        self.csr_access(p, |csr| { !self.rs1() as RegT & csr }, true, self.rs1() != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b00000000000000000000000001110011")]
#[derive(Debug)]
struct ECALL(InsnT);

impl Execution for ECALL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        match p.state().privilege() {
            Privilege::M => Err(Exception::MCall),
            Privilege::S => Err(Exception::SCall),
            Privilege::U => Err(Exception::UCall),
        }
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b00010000001000000000000001110011")]
#[derive(Debug)]
struct SRET(InsnT);

impl Execution for SRET {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_privilege_level(Privilege::S)?;
        let csrs = p.state().csrs::<ICsrs>().unwrap();
        let spp = csrs.mstatus().spp();
        let spie = csrs.mstatus().spie();
        csrs.mstatus_mut().set_sie(spie);
        csrs.mstatus_mut().set_spie(1);
        let u_value: u8 = Privilege::U.into();
        csrs.mstatus_mut().set_spp(u_value as RegT);
        p.state().set_privilege(Privilege::try_from(spp as u8).unwrap());
        p.state().set_pc(csrs.sepc().get());
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b00110000001000000000000001110011")]
#[derive(Debug)]
struct MRET(InsnT);

impl Execution for MRET {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csrs = p.state().csrs::<ICsrs>().unwrap();
        let mpp = csrs.mstatus().mpp();
        let mpie = csrs.mstatus().mpie();
        csrs.mstatus_mut().set_mie(mpie);
        csrs.mstatus_mut().set_mpie(1);
        let u_value: u8 = Privilege::U.into();
        csrs.mstatus_mut().set_mpp(u_value as RegT);
        p.state().set_privilege(Privilege::try_from(mpp as u8).unwrap());
        p.state().set_pc(csrs.mepc().get());
        Ok(())
    }
}