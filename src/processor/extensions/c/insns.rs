use crate::processor::insn_define::*;
use std::num::Wrapping;
use crate::processor::extensions::f::float::FloatInsn;
use crate::processor::extensions::f::{FRegT, FLen};

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????010???????????10")]
#[derive(Debug)]
struct CLWSP(InsnT);

impl Execution for CLWSP {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd() == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5: RegT = self.imm().bit_range(5, 5);
        let offset_4_2: RegT = self.imm().bit_range(4, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_4_2 << 2 | offset_5 << 5 | offset_7_6 << 6);
        let data = p.load_store().load_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????011???????????10")]
#[derive(Debug)]
struct CLDSPCFLWSP(InsnT);

impl CLDSPCFLWSP {
    fn execute_c_ldsp(&self, p: &Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        if self.rd() == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        let offset_8_6: RegT = self.imm().bit_range(2, 0);
        let offset_5: RegT = self.imm().bit_range(5, 5);
        let offset_4_3: RegT = self.imm().bit_range(4, 3);
        let offset: Wrapping<RegT> = Wrapping(offset_4_3 << 3 | offset_5 << 5 | offset_8_6 << 6);
        let data = p.load_store().load_double_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data as RegT & p.state().config().xlen.mask());
        Ok(())
    }
    fn execute_c_lwsp(&self, p: &Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5: RegT = self.imm().bit_range(5, 5);
        let offset_4_2: RegT = self.imm().bit_range(4, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_4_2 << 2 | offset_5 << 5 | offset_7_6 << 6);
        let data = p.load_store().load_word((base + offset).0, p.mmu())?;
        f.set_freg(self.rd() as RegT, f.flen.padding(data as FRegT, FLen::F32));
        Ok(())
    }
}

impl FloatInsn for CLDSPCFLWSP {}

impl Execution for CLDSPCFLWSP {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_ldsp(p, base)?;
        } else {
            self.execute_c_lwsp(p, base)?;
        }
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????001???????????10")]
#[derive(Debug)]
struct CFLDSP(InsnT);

impl FloatInsn for CFLDSP {}

impl Execution for CFLDSP {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        let offset_8_6: RegT = self.imm().bit_range(2, 0);
        let offset_5: RegT = self.imm().bit_range(5, 5);
        let offset_4_3: RegT = self.imm().bit_range(4, 3);
        let offset: Wrapping<RegT> = Wrapping(offset_4_3 << 3 | offset_5 << 5 | offset_8_6 << 6);
        let data = p.load_store().load_double_word((base + offset).0, p.mmu())?;
        f.set_freg(self.rd() as RegT, f.flen.padding(data as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CSS)]
#[code("0b????????????????110???????????10")]
#[derive(Debug)]
struct CSWSP(InsnT);

impl Execution for CSWSP {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5_2: RegT = self.imm().bit_range(5, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_5_2 << 2 | offset_7_6 << 6);
        let src = p.state().xreg(self.rs2() as RegT);
        p.load_store().store_word((base + offset).0, src, p.mmu())?;
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CSS)]
#[code("0b????????????????111???????????10")]
#[derive(Debug)]
struct CSDSPFSWSP(InsnT);

impl CSDSPFSWSP {
    fn execute_c_sdsp(&self, p: &Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let offset_8_6: RegT = self.imm().bit_range(2, 0);
        let offset_5_3: RegT = self.imm().bit_range(5, 3);
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_8_6 << 6);
        let src = p.state().xreg(self.rs2() as RegT);
        p.load_store().store_double_word((base + offset).0, src, p.mmu())
    }
    fn execute_c_fswsp(&self, p: &Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5_2: RegT = self.imm().bit_range(5, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_5_2 << 2 | offset_7_6 << 6);
        let src = f.freg(self.rs2() as RegT) as RegT;
        p.load_store().store_word((base + offset).0, src, p.mmu())
    }
}

impl FloatInsn for CSDSPFSWSP {}

impl Execution for CSDSPFSWSP {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_sdsp(p, base)?;
        } else {
            self.execute_c_fswsp(p, base)?;
        };
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CSS)]
#[code("0b????????????????101???????????10")]
#[derive(Debug)]
struct CFSDSP(InsnT);

impl FloatInsn for CFSDSP {}

impl Execution for CFSDSP {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        let offset_8_6: RegT = self.imm().bit_range(2, 0);
        let offset_5_3: RegT = self.imm().bit_range(5, 3);
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_8_6 << 6);
        let src = f.freg(self.rs2() as RegT) as RegT;
        p.load_store().store_double_word((base + offset).0, src, p.mmu())?;
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CL)]
#[code("0b????????????????010???????????00")]
#[derive(Debug)]
struct CLW(InsnT);

impl Execution for CLW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset_6: RegT = self.imm().bit_range(0, 0);
        let offset_5_3: RegT = self.imm().bit_range(4, 2);
        let offset_2: RegT = self.imm().bit_range(1, 1);
        let offset: Wrapping<RegT> = Wrapping(offset_2 << 2 | offset_5_3 << 3 | offset_6 << 6);
        let data = p.load_store().load_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CL)]
#[code("0b????????????????011???????????00")]
#[derive(Debug)]
struct CLDFLW(InsnT);

impl CLDFLW {
    fn execute_c_ld(&self, p: &Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5_3: RegT = self.imm().bit_range(4, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_7_6 << 6);
        let data = p.load_store().load_double_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data as RegT & p.state().config().xlen.mask());
        Ok(())
    }
    fn execute_c_flw(&self, p: &Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let offset_6: RegT = self.imm().bit_range(0, 0);
        let offset_5_3: RegT = self.imm().bit_range(4, 2);
        let offset_2: RegT = self.imm().bit_range(1, 1);
        let offset: Wrapping<RegT> = Wrapping(offset_2 << 2 | offset_5_3 << 3 | offset_6 << 6);
        let data = p.load_store().load_word((base + offset).0, p.mmu())?;
        f.set_freg(self.rd() as RegT, f.flen.padding(data as FRegT, FLen::F32));
        Ok(())
    }
}

impl FloatInsn for CLDFLW {}

impl Execution for CLDFLW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_ld(p, base)?;
        } else {
            self.execute_c_flw(p, base)?;
        };
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CL)]
#[code("0b????????????????001???????????00")]
#[derive(Debug)]
struct CFLD(InsnT);

impl FloatInsn for CFLD {}

impl Execution for CFLD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5_3: RegT = self.imm().bit_range(4, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_7_6 << 6);
        let data = p.load_store().load_double_word((base + offset).0, p.mmu())?;
        f.set_freg(self.rd() as RegT, f.flen.padding(data as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CS)]
#[code("0b????????????????110???????????00")]
#[derive(Debug)]
struct CSW(InsnT);

impl Execution for CSW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset_6: RegT = self.imm().bit_range(0, 0);
        let offset_5_3: RegT = self.imm().bit_range(4, 2);
        let offset_2: RegT = self.imm().bit_range(1, 1);
        let offset: Wrapping<RegT> = Wrapping(offset_2 << 2 | offset_5_3 << 3 | offset_6 << 6);
        let src = p.state().xreg(self.rs2() as RegT);
        p.load_store().store_word((base + offset).0, src, p.mmu())?;
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CS)]
#[code("0b????????????????111???????????00")]
#[derive(Debug)]
struct CSDFSW(InsnT);

impl CSDFSW {
    fn execute_c_sd(&self, p: &Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5_3: RegT = self.imm().bit_range(4, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_7_6 << 6);
        let src = p.state().xreg(self.rs2() as RegT);
        p.load_store().store_double_word((base + offset).0, src, p.mmu())
    }
    fn execute_c_fsw(&self, p: &Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let offset_6: RegT = self.imm().bit_range(0, 0);
        let offset_5_3: RegT = self.imm().bit_range(4, 2);
        let offset_2: RegT = self.imm().bit_range(1, 1);
        let offset: Wrapping<RegT> = Wrapping(offset_2 << 2 | offset_5_3 << 3 | offset_6 << 6);
        let src = f.freg(self.rs2() as RegT) as RegT;
        p.load_store().store_word((base + offset).0, src, p.mmu())
    }
}

impl FloatInsn for CSDFSW {}

impl Execution for CSDFSW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_sd(p, base)?;
        } else {
            self.execute_c_fsw(p, base)?;
        };
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CS)]
#[code("0b????????????????101???????????00")]
#[derive(Debug)]
struct CFSD(InsnT);

impl FloatInsn for CFSD {}

impl Execution for CFSD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5_3: RegT = self.imm().bit_range(4, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_7_6 << 6);
        let src = f.freg(self.rs2() as RegT) as RegT;
        p.load_store().store_double_word((base + offset).0, src, p.mmu())?;
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

trait CJump: InstructionImp {
    fn jump(&self, p: &Processor) -> Result<(), Exception> {
        let offset_3_1: RegT = self.imm().bit_range(3, 1);
        let offset_4: RegT = self.imm().bit_range(9, 9);
        let offset_5: RegT = self.imm().bit_range(0, 0);
        let offset_6: RegT = self.imm().bit_range(5, 5);
        let offset_7: RegT = self.imm().bit_range(4, 4);
        let offset_9_8: RegT = self.imm().bit_range(8, 7);
        let offset_10: RegT = self.imm().bit_range(6, 6);
        let offset_11: RegT = self.imm().bit_range(10, 10);
        let offset: Wrapping<RegT> = Wrapping(sext(offset_3_1 << 1 | offset_4 << 4 | offset_5 << 5 | offset_6 << 6 | offset_7 << 7 | offset_9_8 << 8 | offset_10 << 10 | offset_11 << 11, self.imm_len() + 1));
        let t = (Wrapping(p.state().pc()) + offset).0;
        if t.trailing_zeros() < 1 {
            return Err(Exception::FetchMisaligned(t));
        }
        p.state().set_pc(t);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CJ)]
#[code("0b????????????????101???????????01")]
#[derive(Debug)]
struct CJ(InsnT);

impl CJump for CJ {}

impl Execution for CJ {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        self.jump(p)
    }
}

#[derive(Instruction)]
#[format(CJ)]
#[code("0b????????????????001???????????01")]
#[derive(Debug)]
struct CJALADDIW(InsnT);

impl CJALADDIW {
    fn execute_c_jal(&self, p: &Processor) -> Result<(), Exception> {
        self.jump(p)?;
        p.state().set_xreg(1, p.state().pc() + 2);
        Ok(())
    }
    fn execute_c_addiw(&self, p: &Processor) -> Result<(), Exception> {
        let rs1_addr: RegT = self.ir().bit_range(11, 7);
        let rd_addr: RegT = self.ir().bit_range(11, 7);
        let imm_1: RegT = self.ir().bit_range(6, 2);
        let imm_2: RegT = self.ir().bit_range(12, 12);
        let imm: RegT = imm_1 | imm_2 << 5;

        if rd_addr == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        let rs1: Wrapping<RegT> = Wrapping(sext(p.state().xreg(rs1_addr), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(imm, 6));
        p.state().set_xreg(rd_addr, sext((rs1 + rs2).0, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

impl CJump for CJALADDIW {}

impl Execution for CJALADDIW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_addiw(p)?;
        } else {
            self.execute_c_jal(p)?;
        };
        Ok(())
    }
}


trait CJumpR: InstructionImp {
    fn jump(&self, p: &Processor) -> Result<(), Exception> {
        if self.rs1() == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        let t = p.state().xreg(self.rs1() as RegT);
        if t.trailing_zeros() < 1 {
            return Err(Exception::FetchMisaligned(t));
        }
        p.state().set_pc(t);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CR)]
#[code("0b????????????????1000?????0000010")]
#[derive(Debug)]
struct CJR(InsnT);

impl CJumpR for CJR {}

impl Execution for CJR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        self.jump(p)
    }
}

#[derive(Instruction)]
#[format(CR)]
#[code("0b????????????????1001?????0000010")]
#[derive(Debug)]
struct CJALR(InsnT);

impl CJumpR for CJALR {}

impl Execution for CJALR {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        self.jump(p)?;
        p.state().set_xreg(1, p.state().pc() + 2);
        Ok(())
    }
}


trait CBranch: InstructionImp {
    fn branch<F: Fn(RegT) -> bool>(&self, p: &Processor, condition: F) -> Result<(), Exception> {
        let offset_2_1: RegT = self.imm().bit_range(2, 1);
        let offset_4_3: RegT = self.imm().bit_range(6, 5);
        let offset_5: RegT = self.imm().bit_range(0, 0);
        let offset_7_6: RegT = self.imm().bit_range(4, 3);
        let offset_8: RegT = self.imm().bit_range(7, 7);

        let offset: Wrapping<RegT> = Wrapping(sext(offset_2_1 << 1 | offset_4_3 << 3 | offset_5 << 5 | offset_7_6 << 6 | offset_8 << 8, self.imm_len() + 1));
        let pc: Wrapping<RegT> = Wrapping(p.state().pc());
        let rs1 = p.state().xreg(self.rs1() as RegT);
        if condition(rs1) {
            let t = (offset + pc).0;
            if t.trailing_zeros() < 1 {
                return Err(Exception::FetchMisaligned(t));
            }
            p.state().set_pc(t);
        } else {
            p.state().set_pc(pc.0 + 2);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????110???????????01")]
#[derive(Debug)]
struct CBEQZ(InsnT);

impl CBranch for CBEQZ {}

impl Execution for CBEQZ {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        self.branch(p, |rs1| { rs1 == 0 })
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????111???????????01")]
#[derive(Debug)]
struct CBNEZ(InsnT);

impl CBranch for CBNEZ {}

impl Execution for CBNEZ {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        self.branch(p, |rs1| { rs1 != 0 })
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????010???????????01")]
#[derive(Debug)]
struct CLI(InsnT);

impl Execution for CLI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd() == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        p.state().set_xreg(self.rd() as RegT, sext(self.imm() as RegT, self.imm_len()) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????011???????????01")]
#[derive(Debug)]
struct CLUIADDI16SP(InsnT);

impl CLUIADDI16SP {
    fn execute_c_lui(&self, p: &Processor) -> Result<(), Exception> {
        p.state().set_xreg(self.rd() as RegT, sext((self.imm() as RegT) << (12 as RegT), self.imm_len() + 12) & p.state().config().xlen.mask());
        Ok(())
    }
    fn execute_c_addi16sp(&self, p: &Processor) -> Result<(), Exception> {
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        let imm_4: RegT = self.imm().bit_range(4, 4);
        let imm_5: RegT = self.imm().bit_range(0, 0);
        let imm_6: RegT = self.imm().bit_range(3, 3);
        let imm_8_7: RegT = self.imm().bit_range(2, 1);
        let imm_9: RegT = self.imm().bit_range(5, 5);
        let rs2: Wrapping<RegT> = Wrapping(sext(imm_4 << 4 | imm_5 << 5 | imm_6 << 6 | imm_8_7 << 7 | imm_9 << 9, self.imm_len() + 4));
        p.state().set_xreg(2, (rs1 + rs2).0 & p.state().config().xlen.mask());
        Ok(())
    }
}

impl Execution for CLUIADDI16SP {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd() == 0 || self.imm() == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        if self.rd() == 2 {
            self.execute_c_addi16sp(p)?
        } else {
            self.execute_c_lui(p)?
        }
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????000???????????01")]
#[derive(Debug)]
struct CADDI(InsnT);

impl Execution for CADDI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd() == 0 || self.imm() == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let rs2: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        p.state().set_xreg(self.rd() as RegT, (rs1 + rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CIW)]
#[code("0b????????????????000???????????00")]
#[derive(Debug)]
struct CADDI14SPN(InsnT);

impl Execution for CADDI14SPN {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.imm() == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        let imm_2: RegT = self.imm().bit_range(1, 1);
        let imm_3: RegT = self.imm().bit_range(0, 0);
        let imm_5_4: RegT = self.imm().bit_range(7, 6);
        let imm_9_6: RegT = self.imm().bit_range(5, 2);
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        let rs2: Wrapping<RegT> = Wrapping(imm_2 << 2 | imm_3 << 3 | imm_5_4 << 4 | imm_9_6 << 6);
        p.state().set_xreg(self.rd() as RegT, (rs1 + rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????000???????????10")]
#[derive(Debug)]
struct CSLLI(InsnT);

impl Execution for CSLLI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd() == 0 || self.imm() == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        if let Err(_) = p.state().check_xlen(XLen::X64) {
            if self.imm() & (1 << 5) != 0 {
                return Err(Exception::IllegalInsn(self.ir()));
            }
        }
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let shamt: RegT = (self.imm() as RegT).bit_range(p.state().config().xlen.len().trailing_zeros() as usize - 1, 0);
        p.state().set_xreg(self.rd() as RegT, rs1.wrapping_shl(shamt as u32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????100?00????????01")]
#[derive(Debug)]
struct CSRLI(InsnT);

impl Execution for CSRLI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let shamt_4_0:RegT = self.imm().bit_range(4,0);
        let shamt_5:RegT = self.imm().bit_range(7,7);
        let shamt = shamt_4_0 | shamt_5 << 5;
        if self.rd() == 0 || shamt == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        if let Err(_) = p.state().check_xlen(XLen::X64) {
            if shamt_5 != 0 {
                return Err(Exception::IllegalInsn(self.ir()));
            }
        }
        let rs1 = p.state().xreg(self.rs1() as RegT);
        p.state().set_xreg(self.rd() as RegT, rs1 >> shamt);
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????100?01????????01")]
#[derive(Debug)]
struct CSRAI(InsnT);

impl Execution for CSRAI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let shamt_4_0:RegT = self.imm().bit_range(4,0);
        let shamt_5:RegT = self.imm().bit_range(7,7);
        let shamt = shamt_4_0 | shamt_5 << 5;
        if self.rd() == 0 || shamt == 0 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        if let Err(_) = p.state().check_xlen(XLen::X64) {
            if shamt_5 != 0 {
                return Err(Exception::IllegalInsn(self.ir()));
            }
        }
        let rs1 = p.state().xreg(self.rs1() as RegT);
        p.state().set_xreg(self.rd() as RegT, sext(rs1.wrapping_shr(shamt as u32), p.state().config().xlen.len() - shamt as usize) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????100?10????????01")]
#[derive(Debug)]
struct CANDI(InsnT);

impl Execution for CANDI {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let imm_4_0:RegT = self.imm().bit_range(4,0);
        let imm_5:RegT = self.imm().bit_range(7,7);
        let imm = imm_4_0 | imm_5 << 5;
        let rs1 = p.state().xreg(self.rs1() as RegT);
        let rs2 = sext(imm, 6) & p.state().config().xlen.mask();
        p.state().set_xreg(self.rd() as RegT, (rs1 & rs2) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 2);
        Ok(())
    }
}

