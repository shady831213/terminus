use crate::prelude::*;
use std::num::Wrapping;
use std::convert::TryFrom;


trait Branch: InstructionImp {
    fn branch<F: Fn(RegT, RegT) -> bool>(&self, p: &Processor, condition: F) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));

        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = p.state().xreg(self.rs2(p.state().ir()));
        if condition(rs1, rs2) {
            let t = (offset + pc).0;
            if let Err(_) = p.state().check_extension('c') {
                if t.trailing_zeros() < 2 {
                    return Err(Exception::FetchMisaligned(t));
                }
            } else if t.trailing_zeros() < 1 {
                return Err(Exception::FetchMisaligned(t));
            }
            p.state().set_pc(t);
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
struct BEQ();

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
struct BNE();

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
struct BLT();

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
struct BGE();

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
struct BLTU();

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
struct BGEU();

impl Branch for BGEU {}

impl Execution for BGEU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.branch(p, |rs1, rs2| { rs1 >= rs2 })
    }
}


trait Jump: InstructionImp {
    fn jump<F: Fn(Wrapping<RegT>) -> Wrapping<RegT>>(&self, p: &Processor, target: F) -> Result<(), Exception> {
        let offset: Wrapping<RegT> = Wrapping(sext(((self.imm(p.state().ir()) >> 1) << 1) as RegT, self.imm_len()));
        let t = target(offset).0;
        if let Err(_) = p.state().check_extension('c') {
            if t.trailing_zeros() < 2 {
                return Err(Exception::FetchMisaligned(t));
            }
        } else if t.trailing_zeros() < 1 {
            return Err(Exception::FetchMisaligned(t));
        }
        p.state().set_pc(t);
        p.state().set_xreg(self.rd(p.state().ir()), p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????1100111")]
#[derive(Debug)]
struct JALR();

impl Jump for JALR {}

impl Execution for JALR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.jump(p, |offset| { offset + Wrapping(p.state().xreg(self.rs1(p.state().ir()))) })
    }
}

#[derive(Instruction)]
#[format(J)]
#[code("0b?????????????????????????1101111")]
#[derive(Debug)]
struct JAL();

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
struct LUI();

impl Execution for LUI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state.set_xreg(self.rd(p.state().ir()), sext(self.imm(p.state().ir()) as RegT, self.imm_len()) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(U)]
#[code("0b?????????????????????????0010111")]
#[derive(Debug)]
struct AUIPC();

impl Execution for AUIPC {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        p.state.set_xreg(self.rd(p.state().ir()), (pc + offset).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0010011")]
#[derive(Debug)]
struct ADDI();

impl Execution for ADDI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 + rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0011011")]
#[derive(Debug)]
struct ADDIW();

impl Execution for ADDIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs1(p.state().ir())), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        p.state().set_xreg(self.rd(p.state().ir()), sext((rs1 + rs2).0, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b000000???????????001?????0010011")]
#[derive(Debug)]
struct SLLI();

impl Execution for SLLI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let high: RegT = (self.imm(p.state().ir()) as RegT) >> (p.state().config().xlen.len().trailing_zeros() as RegT);
        if high != 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let shamt: RegT = (self.imm(p.state().ir()) as RegT) & ((1 << p.state().config().xlen.len().trailing_zeros()) - 1) as RegT;
        p.state().set_xreg(self.rd(p.state().ir()), rs1.wrapping_shl(shamt as u32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0000000??????????001?????0011011")]
#[derive(Debug)]
struct SLLIW();

impl Execution for SLLIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let high: RegT = (self.imm(p.state().ir()) >> 5) as RegT;
        if high != 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let shamt: RegT = (self.imm(p.state().ir()) as RegT) & 0x1f;
        p.state().set_xreg(self.rd(p.state().ir()), sext(rs1.wrapping_shl(shamt as u32), 32));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b000000???????????101?????0010011")]
#[derive(Debug)]
struct SRLI();

impl Execution for SRLI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir())) & p.state().config().xlen.mask();
        let shamt: RegT = (self.imm(p.state().ir()) as RegT) & ((1 << p.state().config().xlen.len().trailing_zeros()) - 1) as RegT;
        p.state().set_xreg(self.rd(p.state().ir()), rs1 >> shamt);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0000000??????????101?????0011011")]
#[derive(Debug)]
struct SRLIW();

impl Execution for SRLIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1(p.state().ir())) as u32 as RegT;
        let shamt: RegT = (self.imm(p.state().ir()) as RegT) & 0x1f;
        p.state().set_xreg(self.rd(p.state().ir()), sext(rs1 >> shamt, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b010000???????????101?????0010011")]
#[derive(Debug)]
struct SRAI();

impl Execution for SRAI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir())) & p.state().config().xlen.mask();
        let shamt: RegT = (self.imm(p.state().ir()) as RegT) & ((1 << p.state().config().xlen.len().trailing_zeros()) - 1) as RegT;
        p.state().set_xreg(self.rd(p.state().ir()), sext(rs1.wrapping_shr(shamt as u32), p.state().config().xlen.len() - shamt as usize) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0100000??????????101?????0011011")]
#[derive(Debug)]
struct SRAIW();

impl Execution for SRAIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1(p.state().ir())) as u32 as RegT;
        let shamt: RegT = (self.imm(p.state().ir()) as RegT) & 0x1f;
        p.state().set_xreg(self.rd(p.state().ir()), sext(rs1.wrapping_shr(shamt as u32), 32 - shamt as usize) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0010011")]
#[derive(Debug)]
struct SLTI();

impl Execution for SLTI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = sext(p.state().xreg(self.rs1(p.state().ir())), p.state.config().xlen.len()) as SRegT;
        let rs2 = sext(self.imm(p.state().ir()) as RegT, self.imm_len()) as SRegT;
        if rs1 < rs2 {
            p.state().set_xreg(self.rd(p.state().ir()), 1)
        } else {
            p.state().set_xreg(self.rd(p.state().ir()), 0)
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0010011")]
#[derive(Debug)]
struct SLTIU();

impl Execution for SLTIU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = sext(self.imm(p.state().ir()) as RegT, self.imm_len()) & p.state().config().xlen.mask();
        if rs1 == 0 && self.rs2(p.state().ir()) == 1 || rs1 < rs2 {
            p.state().set_xreg(self.rd(p.state().ir()), 1)
        } else {
            p.state().set_xreg(self.rd(p.state().ir()), 0)
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????100?????0010011")]
#[derive(Debug)]
struct XORI();

impl Execution for XORI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = sext(self.imm(p.state().ir()) as RegT, self.imm_len()) & p.state().config().xlen.mask();
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 ^ rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????110?????0010011")]
#[derive(Debug)]
struct ORI();

impl Execution for ORI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = sext(self.imm(p.state().ir()) as RegT, self.imm_len()) & p.state().config().xlen.mask();
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 | rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????111?????0010011")]
#[derive(Debug)]
struct ANDI();

impl Execution for ANDI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = sext(self.imm(p.state().ir()) as RegT, self.imm_len()) & p.state().config().xlen.mask();
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 & rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????000?????0110011")]
#[derive(Debug)]
struct ADD();

impl Execution for ADD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs2(p.state().ir())));
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 + rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????000?????0111011")]
#[derive(Debug)]
struct ADDW();

impl Execution for ADDW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs1(p.state().ir())), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs2(p.state().ir())), 32));
        p.state().set_xreg(self.rd(p.state().ir()), sext((rs1 + rs2).0, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0100000??????????000?????0110011")]
#[derive(Debug)]
struct SUB();

impl Execution for SUB {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs2(p.state().ir())));
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 - rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0100000??????????000?????0111011")]
#[derive(Debug)]
struct SUBW();

impl Execution for SUBW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs1(p.state().ir())), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs2(p.state().ir())), 32));
        p.state().set_xreg(self.rd(p.state().ir()), sext((rs1 - rs2).0, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????001?????0110011")]
#[derive(Debug)]
struct SLL();

impl Execution for SLL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let shamt: RegT = p.state().xreg(self.rs2(p.state().ir())) & ((1 << p.state().config().xlen.len().trailing_zeros()) - 1) as RegT;
        p.state().set_xreg(self.rd(p.state().ir()), rs1.wrapping_shl(shamt as u32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????001?????0111011")]
#[derive(Debug)]
struct SLLW();

impl Execution for SLLW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1(p.state().ir())) as u32 as RegT;
        let shamt: RegT = p.state().xreg(self.rs2(p.state().ir())) & 0x1f;
        p.state().set_xreg(self.rd(p.state().ir()), sext(rs1.wrapping_shl(shamt as u32), 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????101?????0110011")]
#[derive(Debug)]
struct SRL();

impl Execution for SRL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir())) & p.state().config().xlen.mask();
        let shamt: RegT = p.state().xreg(self.rs2(p.state().ir())) & ((1 << p.state().config().xlen.len().trailing_zeros()) - 1) as RegT;
        p.state().set_xreg(self.rd(p.state().ir()), rs1 >> shamt);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????101?????0111011")]
#[derive(Debug)]
struct SRLW();

impl Execution for SRLW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1(p.state().ir())) as u32 as RegT;
        let shamt: RegT = p.state().xreg(self.rs2(p.state().ir())) & 0x1f;
        p.state().set_xreg(self.rd(p.state().ir()), sext(rs1 >> shamt, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0100000??????????101?????0110011")]
#[derive(Debug)]
struct SRA();

impl Execution for SRA {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir())) & p.state().config().xlen.mask();
        let shamt: RegT = p.state().xreg(self.rs2(p.state().ir())) & ((1 << p.state().config().xlen.len().trailing_zeros()) - 1) as RegT;
        p.state().set_xreg(self.rd(p.state().ir()), sext(rs1.wrapping_shr(shamt as u32), p.state().config().xlen.len() - shamt as usize) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0100000??????????101?????0111011")]
#[derive(Debug)]
struct SRAW();

impl Execution for SRAW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let rs1: RegT = p.state().xreg(self.rs1(p.state().ir())) as u32 as RegT;
        let shamt: RegT = p.state().xreg(self.rs2(p.state().ir())) & 0x1f;
        p.state().set_xreg(self.rd(p.state().ir()), sext(rs1.wrapping_shr(shamt as u32), 32 - shamt as usize) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????010?????0110011")]
#[derive(Debug)]
struct SLT();

impl Execution for SLT {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = sext(p.state().xreg(self.rs1(p.state().ir())), p.state.config().xlen.len()) as SRegT;
        let rs2 = sext(p.state().xreg(self.rs2(p.state().ir())), p.state.config().xlen.len()) as SRegT;
        if rs1 < rs2 {
            p.state().set_xreg(self.rd(p.state().ir()), 1)
        } else {
            p.state().set_xreg(self.rd(p.state().ir()), 0)
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????011?????0110011")]
#[derive(Debug)]
struct SLTU();

impl Execution for SLTU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = p.state().xreg(self.rs2(p.state().ir()));
        if rs2 != 0 && self.rs1(p.state().ir()) == 0 || rs1 < rs2 {
            p.state().set_xreg(self.rd(p.state().ir()), 1)
        } else {
            p.state().set_xreg(self.rd(p.state().ir()), 0)
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????100?????0110011")]
#[derive(Debug)]
struct XOR();

impl Execution for XOR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = p.state().xreg(self.rs2(p.state().ir()));
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 ^ rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????110?????0110011")]
#[derive(Debug)]
struct OR();

impl Execution for OR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = p.state().xreg(self.rs2(p.state().ir()));
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 | rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????111?????0110011")]
#[derive(Debug)]
struct AND();

impl Execution for AND {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let rs1 = p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = p.state().xreg(self.rs2(p.state().ir()));
        p.state().set_xreg(self.rd(p.state().ir()), (rs1 & rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0000011")]
#[derive(Debug)]
struct LB();

impl Execution for LB {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let data = p.load_store().load_byte(p.state(), (base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd(p.state().ir()), sext(data, 8) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????100?????0000011")]
#[derive(Debug)]
struct LBU();

impl Execution for LBU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let data = p.load_store().load_byte(p.state(), (base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd(p.state().ir()), data);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????001?????0000011")]
#[derive(Debug)]
struct LH();

impl Execution for LH {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let data = p.load_store().load_half_word(p.state(), (base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd(p.state().ir()), sext(data, 16) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????101?????0000011")]
#[derive(Debug)]
struct LHU();

impl Execution for LHU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let data = p.load_store().load_half_word(p.state(), (base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd(p.state().ir()), data);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0000011")]
#[derive(Debug)]
struct LW();

impl Execution for LW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let data = p.load_store().load_word(p.state(), (base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd(p.state().ir()), sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????110?????0000011")]
#[derive(Debug)]
struct LWU();

impl Execution for LWU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let data = p.load_store().load_word(p.state(), (base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd(p.state().ir()), data);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0000011")]
#[derive(Debug)]
struct LD();

impl Execution for LD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let data = p.load_store().load_double_word(p.state(), (base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd(p.state().ir()), data);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait Store: InstructionImp {
    fn offset(&self, code: InsnT) -> Wrapping<RegT> {
        let high: RegT = (self.imm(code) >> 5) as RegT;
        let low = self.rd(code) as RegT;
        Wrapping(sext(high << 5 | low, self.imm_len()))
    }
    fn src(&self, code: InsnT) -> InsnT {
        self.imm(code) & 0x1f
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0100011")]
#[derive(Debug)]
struct SB();

impl Store for SB {}

impl Execution for SB {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let data = p.state().xreg(self.src(p.state().ir()));
        p.load_store.store_byte(p.state(), (base + self.offset(p.state().ir())).0, data, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????001?????0100011")]
#[derive(Debug)]
struct SH();

impl Store for SH {}

impl Execution for SH {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let data = p.state().xreg(self.src(p.state().ir()));
        p.load_store.store_half_word(p.state(), (base + self.offset(p.state().ir())).0, data, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0100011")]
#[derive(Debug)]
struct SW();

impl Store for SW {}

impl Execution for SW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let data = p.state().xreg(self.src(p.state().ir()));
        p.load_store.store_word(p.state(), (base + self.offset(p.state().ir())).0, data, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0100011")]
#[derive(Debug)]
struct SD();

impl Store for SD {}

impl Execution for SD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1(p.state().ir())));
        let data = p.state().xreg(self.src(p.state().ir()));
        p.load_store.store_double_word(p.state(), (base + self.offset(p.state().ir())).0, data, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????000?????0001111")]
#[derive(Debug)]
struct FENCE();

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
struct FENCEI();

impl Execution for FENCEI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.fetcher().flush_icache();
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait CsrAccess: InstructionImp {
    fn csr_access<F: Fn(RegT) -> RegT>(&self, p: &Processor, csr_value: F, read_csr: bool, write_csr: bool) -> Result<(), Exception> {
        let csr = if read_csr {
            p.state().csr(self.imm(p.state().ir()))?
        } else {
            0
        };
        if write_csr {
            p.state().set_csr(self.imm(p.state().ir()), csr_value(csr))?;
        }
        p.state().set_xreg(self.rd(p.state().ir()), csr);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????001?????1110011")]
#[derive(Debug)]
struct CSRRW();

impl CsrAccess for CSRRW {}

impl Execution for CSRRW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |_| { p.state().xreg(self.rs1(p.state().ir())) }, self.rd(p.state().ir()) != 0, true)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????1110011")]
#[derive(Debug)]
struct CSRRS();

impl CsrAccess for CSRRS {}

impl Execution for CSRRS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |csr| { p.state().xreg(self.rs1(p.state().ir())) | csr }, true, self.rs1(p.state().ir()) != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????1110011")]
#[derive(Debug)]
struct CSRRC();

impl CsrAccess for CSRRC {}

impl Execution for CSRRC {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |csr| { !p.state().xreg(self.rs1(p.state().ir())) & csr }, true, self.rs1(p.state().ir()) != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????101?????1110011")]
#[derive(Debug)]
struct CSRRWI();

impl CsrAccess for CSRRWI {}

impl Execution for CSRRWI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |_| { self.rs1(p.state().ir()) as RegT }, self.rd(p.state().ir()) != 0, true)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????110?????1110011")]
#[derive(Debug)]
struct CSRRSI();

impl CsrAccess for CSRRSI {}

impl Execution for CSRRSI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |csr| { self.rs1(p.state().ir()) as RegT | csr }, true, self.rs1(p.state().ir()) != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????111?????1110011")]
#[derive(Debug)]
struct CSRRCI();

impl CsrAccess for CSRRCI {}

impl Execution for CSRRCI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        self.csr_access(p, |csr| { !(self.rs1(p.state().ir()) as RegT) & csr }, true, self.rs1(p.state().ir()) != 0)
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b00000000000000000000000001110011")]
#[derive(Debug)]
struct ECALL();

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
#[code("0b00000000000100000000000001110011")]
#[derive(Debug)]
struct EBREAK();

impl Execution for EBREAK {
    fn execute(&self, _: &Processor) -> Result<(), Exception> {
        return Err(Exception::Breakpoint);
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b00110000001000000000000001110011")]
#[derive(Debug)]
struct MRET();

impl Execution for MRET {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csrs = p.state().icsrs();
        let mpp = csrs.mstatus().mpp();
        let mpie = csrs.mstatus().mpie();
        csrs.mstatus_mut().set_mie(mpie);
        csrs.mstatus_mut().set_mpie(1);
        let u_value: u8 = Privilege::U.into();
        csrs.mstatus_mut().set_mpp(u_value as RegT);
        p.state().set_privilege(Privilege::try_from(mpp as u8).unwrap());
        p.mmu().flush_tlb();
        p.fetcher().flush_icache();
        if p.state().check_extension('c').is_err() {
            p.state().set_pc((csrs.mepc().get() >> 2) << 2);
        } else {
            p.state().set_pc(csrs.mepc().get());
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b00010000010100000000000001110011")]
#[derive(Debug)]
struct WFI();

impl Execution for WFI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let csrs = p.state().icsrs();
        if csrs.mstatus().tw() != 0 && p.state().config().privilege_level() != PrivilegeLevel::M {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        if csrs.mip().get() != 0 {
            p.state().set_pc(p.state().pc() + 4);
        }
        Ok(())
    }
}