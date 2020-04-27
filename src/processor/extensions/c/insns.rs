use crate::prelude::*;
use std::num::Wrapping;
use crate::processor::extensions::f::float::FloatInsn;
use crate::processor::extensions::f::{FRegT, FLen};

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????010???????????10")]
#[derive(Debug)]
struct CLWSP();

impl Execution for CLWSP {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(2));
        let offset_7_6: RegT = (self.imm(p.state().ir()) & 0x3) as RegT;
        let offset_5: RegT = ((self.imm(p.state().ir()) >> 5) & 0x1) as RegT;
        let offset_4_2: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_4_2 << 2 | offset_5 << 5 | offset_7_6 << 6);
        let data = p.load_store().load_word(p.state(), (base + offset).0, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = sext(data, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????011???????????10")]
#[derive(Debug)]
struct CLDSPCFLWSP();

impl CLDSPCFLWSP {
    fn execute_c_ldsp(&self, p: &mut Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        if self.rd(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let offset_8_6: RegT = (self.imm(p.state().ir()) & 0x7) as RegT;
        let offset_5: RegT = ((self.imm(p.state().ir()) >> 5) & 0x1) as RegT;
        let offset_4_3: RegT = ((self.imm(p.state().ir()) >> 3) & 0x3) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_4_3 << 3 | offset_5 << 5 | offset_8_6 << 6);
        let data = p.load_store().load_double_word(p.state(), (base + offset).0, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data as RegT & p.state().config().xlen.mask();
        p.state_mut().set_xreg(rd, value);
        Ok(())
    }
    fn execute_c_lwsp(&self, p: &mut Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let offset_7_6: RegT = (self.imm(p.state().ir()) & 0x3) as RegT;
        let offset_5: RegT = ((self.imm(p.state().ir()) >> 5) & 0x1) as RegT;
        let offset_4_2: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_4_2 << 2 | offset_5 << 5 | offset_7_6 << 6);
        let data = p.load_store().load_word(p.state(), (base + offset).0, p.mmu())?;
        f.set_freg(self.rd(p.state().ir()), f.flen.padding(data as FRegT, FLen::F32));
        Ok(())
    }
}

impl FloatInsn for CLDSPCFLWSP {}

impl Execution for CLDSPCFLWSP {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(2));
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_ldsp(p, base)?;
        } else {
            self.execute_c_lwsp(p, base)?;
        }
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????001???????????10")]
#[derive(Debug)]
struct CFLDSP();

impl FloatInsn for CFLDSP {}

impl Execution for CFLDSP {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(2));
        let offset_8_6: RegT = (self.imm(p.state().ir()) & 0x7) as RegT;
        let offset_5: RegT = ((self.imm(p.state().ir()) >> 5) & 0x1) as RegT;
        let offset_4_3: RegT = ((self.imm(p.state().ir()) >> 3) & 0x3) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_4_3 << 3 | offset_5 << 5 | offset_8_6 << 6);
        let data = p.load_store().load_double_word(p.state(), (base + offset).0, p.mmu())?;
        f.set_freg(self.rd(p.state().ir()), f.flen.padding(data as FRegT, FLen::F64));
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CSS)]
#[code("0b????????????????110???????????10")]
#[derive(Debug)]
struct CSWSP();

impl Execution for CSWSP {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(2));
        let offset_7_6: RegT = (self.imm(p.state().ir()) & 0x3) as RegT;
        let offset_5_2: RegT = ((self.imm(p.state().ir()) >> 2) & 0xf) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_5_2 << 2 | offset_7_6 << 6);
        let src = *p.state().xreg(self.rs2(p.state().ir()));
        p.load_store().store_word(p.state(), (base + offset).0, src, p.mmu())?;
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CSS)]
#[code("0b????????????????111???????????10")]
#[derive(Debug)]
struct CSDSPFSWSP();

impl CSDSPFSWSP {
    fn execute_c_sdsp(&self, p: &mut Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let offset_8_6: RegT = (self.imm(p.state().ir()) & 0x7) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 3) & 0x7) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_8_6 << 6);
        let src = *p.state().xreg(self.rs2(p.state().ir()));
        p.load_store().store_double_word(p.state(), (base + offset).0, src, p.mmu())
    }
    fn execute_c_fswsp(&self, p: &mut Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let offset_7_6: RegT = (self.imm(p.state().ir()) & 0x3) as RegT;
        let offset_5_2: RegT = ((self.imm(p.state().ir()) >> 2) & 0xf) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_5_2 << 2 | offset_7_6 << 6);
        let src = f.freg(self.rs2(p.state().ir())) as RegT;
        p.load_store().store_word(p.state(), (base + offset).0, src, p.mmu())
    }
}

impl FloatInsn for CSDSPFSWSP {}

impl Execution for CSDSPFSWSP {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(2));
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_sdsp(p, base)?;
        } else {
            self.execute_c_fswsp(p, base)?;
        };
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CSS)]
#[code("0b????????????????101???????????10")]
#[derive(Debug)]
struct CFSDSP();

impl FloatInsn for CFSDSP {}

impl Execution for CFSDSP {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(2));
        let offset_8_6: RegT = (self.imm(p.state().ir()) & 0x7) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 3) & 0x7) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_8_6 << 6);
        let src = f.freg(self.rs2(p.state().ir())) as RegT;
        p.load_store().store_double_word(p.state(), (base + offset).0, src, p.mmu())?;
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CL)]
#[code("0b????????????????010???????????00")]
#[derive(Debug)]
struct CLW();

impl Execution for CLW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let offset_6: RegT = (self.imm(p.state().ir()) & 0x1) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset_2: RegT = ((self.imm(p.state().ir()) >> 1) & 0x1) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_2 << 2 | offset_5_3 << 3 | offset_6 << 6);
        let data = p.load_store().load_word(p.state(), (base + offset).0, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = sext(data, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CL)]
#[code("0b????????????????011???????????00")]
#[derive(Debug)]
struct CLDFLW();

impl CLDFLW {
    fn execute_c_ld(&self, p: &mut Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let offset_7_6: RegT = (self.imm(p.state().ir()) & 0x3) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_7_6 << 6);
        let data = p.load_store().load_double_word(p.state(), (base + offset).0, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data as RegT & p.state().config().xlen.mask();
        p.state_mut().set_xreg(rd, value);
        Ok(())
    }
    fn execute_c_flw(&self, p: &mut Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let offset_6: RegT = (self.imm(p.state().ir()) & 0x1) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset_2: RegT = ((self.imm(p.state().ir()) >> 1) & 0x1) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_2 << 2 | offset_5_3 << 3 | offset_6 << 6);
        let data = p.load_store().load_word(p.state(), (base + offset).0, p.mmu())?;
        f.set_freg(self.rd(p.state().ir()), f.flen.padding(data as FRegT, FLen::F32));
        Ok(())
    }
}

impl FloatInsn for CLDFLW {}

impl Execution for CLDFLW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_ld(p, base)?;
        } else {
            self.execute_c_flw(p, base)?;
        };
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CL)]
#[code("0b????????????????001???????????00")]
#[derive(Debug)]
struct CFLD();

impl FloatInsn for CFLD {}

impl Execution for CFLD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let offset_7_6: RegT = (self.imm(p.state().ir()) & 0x3) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_7_6 << 6);
        let data = p.load_store().load_double_word(p.state(), (base + offset).0, p.mmu())?;
        f.set_freg(self.rd(p.state().ir()), f.flen.padding(data as FRegT, FLen::F64));
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CS)]
#[code("0b????????????????110???????????00")]
#[derive(Debug)]
struct CSW();

impl Execution for CSW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let offset_6: RegT = (self.imm(p.state().ir()) & 0x1) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset_2: RegT = ((self.imm(p.state().ir()) >> 1) & 0x1) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_2 << 2 | offset_5_3 << 3 | offset_6 << 6);
        let src = *p.state().xreg(self.rs2(p.state().ir()));
        p.load_store().store_word(p.state(), (base + offset).0, src, p.mmu())?;
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CS)]
#[code("0b????????????????111???????????00")]
#[derive(Debug)]
struct CSDFSW();

impl CSDFSW {
    fn execute_c_sd(&self, p: &mut Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let offset_7_6: RegT = (self.imm(p.state().ir()) & 0x3) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_7_6 << 6);
        let src = *p.state().xreg(self.rs2(p.state().ir()));
        p.load_store().store_double_word(p.state(), (base + offset).0, src, p.mmu())
    }
    fn execute_c_fsw(&self, p: &mut Processor, base: Wrapping<RegT>) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let offset_6: RegT = (self.imm(p.state().ir()) & 0x1) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset_2: RegT = ((self.imm(p.state().ir()) >> 1) & 0x1) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_2 << 2 | offset_5_3 << 3 | offset_6 << 6);
        let src = f.freg(self.rs2(p.state().ir())) as RegT;
        p.load_store().store_word(p.state(), (base + offset).0, src, p.mmu())
    }
}

impl FloatInsn for CSDFSW {}

impl Execution for CSDFSW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        if let Ok(_) = p.state().check_xlen(XLen::X64) {
            self.execute_c_sd(p, base)?;
        } else {
            self.execute_c_fsw(p, base)?;
        };
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CS)]
#[code("0b????????????????101???????????00")]
#[derive(Debug)]
struct CFSD();

impl FloatInsn for CFSD {}

impl Execution for CFSD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let offset_7_6: RegT = (self.imm(p.state().ir()) & 0x3) as RegT;
        let offset_5_3: RegT = ((self.imm(p.state().ir()) >> 2) & 0x7) as RegT;
        let offset: Wrapping<RegT> = Wrapping(offset_5_3 << 3 | offset_7_6 << 6);
        let src = f.freg(self.rs2(p.state().ir())) as RegT;
        p.load_store().store_double_word(p.state(), (base + offset).0, src, p.mmu())?;
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

trait CJump: InstructionImp {
    fn jump(&self, p: &mut Processor) -> Result<(), Exception> {
        let offset_3_1: RegT = ((self.imm(p.state().ir()) >> 1) & 0x7) as RegT;
        let offset_4: RegT = ((self.imm(p.state().ir()) >> 9) & 0x1) as RegT;
        let offset_5: RegT = (self.imm(p.state().ir()) & 0x1) as RegT;
        let offset_6: RegT = ((self.imm(p.state().ir()) >> 5) & 0x1) as RegT;
        let offset_7: RegT = ((self.imm(p.state().ir()) >> 4) & 0x1) as RegT;
        let offset_9_8: RegT = ((self.imm(p.state().ir()) >> 7) & 0x3) as RegT;
        let offset_10: RegT = ((self.imm(p.state().ir()) >> 6) & 0x1) as RegT;
        let offset_11: RegT = ((self.imm(p.state().ir()) >> 10) & 0x1) as RegT;
        let offset: Wrapping<RegT> = Wrapping(sext(offset_3_1 << 1 | offset_4 << 4 | offset_5 << 5 | offset_6 << 6 | offset_7 << 7 | offset_9_8 << 8 | offset_10 << 10 | offset_11 << 11, self.imm_len() + 1));
        let t = (Wrapping(*p.state().pc()) + offset).0;
        if t.trailing_zeros() < 1 {
            return Err(Exception::FetchMisaligned(t));
        }
        let pc = t;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CJ)]
#[code("0b????????????????101???????????01")]
#[derive(Debug)]
struct CJ();

impl CJump for CJ {}

impl Execution for CJ {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        self.jump(p)
    }
}

#[derive(Instruction)]
#[format(CJ)]
#[code("0b????????????????001???????????01")]
#[derive(Debug)]
struct CJALADDIW();

impl CJALADDIW {
    fn execute_c_jal(&self, p: &mut Processor) -> Result<(), Exception> {
        self.jump(p)?;
        let rd = 1;
        let value = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        Ok(())
    }
    fn execute_c_addiw(&self, p: &mut Processor) -> Result<(), Exception> {
        let rs1_addr = (p.state().ir() >> 7) & 0x1f;
        let rd_addr = (p.state().ir() >> 7) & 0x1f;
        let imm_1: RegT = ((p.state().ir() >> 2) & 0x1f) as RegT;
        let imm_2: RegT = ((p.state().ir() >> 12) & 0x1) as RegT;
        let imm: RegT = imm_1 | imm_2 << 5;

        if rd_addr == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let rs1: Wrapping<RegT> = Wrapping(sext(*p.state().xreg(rs1_addr), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(imm, 6));
        let rd = rd_addr;
        let value = sext((rs1 + rs2).0, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

impl CJump for CJALADDIW {}

impl Execution for CJALADDIW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
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
    fn jump(&self, p: &mut Processor) -> Result<(), Exception> {
        if self.rs1(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let t = *p.state().xreg(self.rs1(p.state().ir()));
        if t.trailing_zeros() < 1 {
            return Err(Exception::FetchMisaligned(t));
        }
        let pc = t;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CR)]
#[code("0b????????????????1000??????????10")]
#[derive(Debug)]
struct CJRMV();

impl CJRMV {
    fn execute_c_jr(&self, p: &mut Processor) -> Result<(), Exception> {
        self.jump(p)
    }
    fn execute_c_mv(&self, p: &mut Processor) -> Result<(), Exception> {
        if self.rd(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let rs2 = *p.state().xreg(self.rs2(p.state().ir()));
        let rd = self.rd(p.state().ir());
        let value = rs2 & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

impl CJumpR for CJRMV {}

impl Execution for CJRMV {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rs2(p.state().ir()) == 0 {
            self.execute_c_jr(p)?
        } else {
            self.execute_c_mv(p)?
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CR)]
#[code("0b????????????????1001??????????10")]
#[derive(Debug)]
struct CJALRADDEBREAK();

impl CJALRADDEBREAK {
    fn execute_c_jalr(&self, p: &mut Processor) -> Result<(), Exception> {
        self.jump(p)?;
        let rd = 1;
        let value = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        Ok(())
    }
    fn execute_c_add(&self, p: &mut Processor) -> Result<(), Exception> {
        if self.rd(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let rs1: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs2(p.state().ir())));
        let rd = self.rd(p.state().ir());
        let value = (rs1 + rs2).0 & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
    fn execute_c_ebreak(&self, _: &Processor) -> Result<(), Exception> {
        Err(Exception::Breakpoint)
    }
}

impl CJumpR for CJALRADDEBREAK {}

impl Execution for CJALRADDEBREAK {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rs2(p.state().ir()) == 0 && self.rd(p.state().ir()) == 0 {
            self.execute_c_ebreak(p)?
        } else if self.rs2(p.state().ir()) == 0 {
            self.execute_c_jalr(p)?
        } else {
            self.execute_c_add(p)?
        }
        Ok(())
    }
}

trait CBranch: InstructionImp {
    fn branch<F: Fn(RegT) -> bool>(&self, p: &mut Processor, condition: F) -> Result<(), Exception> {
        let offset_2_1: RegT = ((self.imm(p.state().ir()) >> 1) & 0x3) as RegT;
        let offset_4_3: RegT = ((self.imm(p.state().ir()) >> 5) & 0x3) as RegT;
        let offset_5: RegT = (self.imm(p.state().ir()) & 0x1) as RegT;
        let offset_7_6: RegT = ((self.imm(p.state().ir()) >> 3) & 0x3) as RegT;
        let offset_8: RegT = ((self.imm(p.state().ir()) >> 7) & 0x1) as RegT;

        let offset: Wrapping<RegT> = Wrapping(sext(offset_2_1 << 1 | offset_4_3 << 3 | offset_5 << 5 | offset_7_6 << 6 | offset_8 << 8, self.imm_len() + 1));
        let pc: Wrapping<RegT> = Wrapping(*p.state().pc());
        let rs1 = *p.state().xreg(self.rs1(p.state().ir()));
        if condition(rs1) {
            let t = (offset + pc).0;
            if t.trailing_zeros() < 1 {
                return Err(Exception::FetchMisaligned(t));
            }
            let pc = t;
            p.state_mut().set_pc(pc);
        } else {
            let pc = pc.0 + 2;
            p.state_mut().set_pc(pc);
        }
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????110???????????01")]
#[derive(Debug)]
struct CBEQZ();

impl CBranch for CBEQZ {}

impl Execution for CBEQZ {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        self.branch(p, |rs1| { rs1 == 0 })
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????111???????????01")]
#[derive(Debug)]
struct CBNEZ();

impl CBranch for CBNEZ {}

impl Execution for CBNEZ {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        self.branch(p, |rs1| { rs1 != 0 })
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????010???????????01")]
#[derive(Debug)]
struct CLI();

impl Execution for CLI {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let rd = self.rd(p.state().ir());
        let value = sext(self.imm(p.state().ir()) as RegT, self.imm_len()) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????011???????????01")]
#[derive(Debug)]
struct CLUIADDI16SP();

impl CLUIADDI16SP {
    fn execute_c_lui(&self, p: &mut Processor) -> Result<(), Exception> {
        let rd = self.rd(p.state().ir());
        let value = sext((self.imm(p.state().ir()) as RegT) << (12 as RegT), self.imm_len() + 12) & p.state().config().xlen.mask();
        p.state_mut().set_xreg(rd, value);
        Ok(())
    }
    fn execute_c_addi16sp(&self, p: &mut Processor) -> Result<(), Exception> {
        let rs1: Wrapping<RegT> = Wrapping(*p.state().xreg(2));
        let imm_4: RegT = ((self.imm(p.state().ir()) >> 4) & 0x1) as RegT;
        let imm_5: RegT = (self.imm(p.state().ir()) & 0x1) as RegT;
        let imm_6: RegT = ((self.imm(p.state().ir()) >> 3) & 0x1) as RegT;
        let imm_8_7: RegT = ((self.imm(p.state().ir()) >> 1) & 0x3) as RegT;
        let imm_9: RegT = ((self.imm(p.state().ir()) >> 5) & 0x1) as RegT;
        let rs2: Wrapping<RegT> = Wrapping(sext(imm_4 << 4 | imm_5 << 5 | imm_6 << 6 | imm_8_7 << 7 | imm_9 << 9, self.imm_len() + 4));
        let rd = 2;
        let value = (rs1 + rs2).0 & p.state().config().xlen.mask();
        p.state_mut().set_xreg(rd, value);
        Ok(())
    }
}

impl Execution for CLUIADDI16SP {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd(p.state().ir()) == 0 || self.imm(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        if self.rd(p.state().ir()) == 2 {
            self.execute_c_addi16sp(p)?
        } else {
            self.execute_c_lui(p)?
        }
        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????000???????????01")]
#[derive(Debug)]
struct CADDINOP();

impl CADDINOP {
    fn execute_c_addi(&self, p: &mut Processor) -> Result<(), Exception> {
        if self.rd(p.state().ir()) == 0 || self.imm(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let rs1: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let rd = self.rd(p.state().ir());
        let value = (rs1 + rs2).0 & p.state().config().xlen.mask();
        p.state_mut().set_xreg(rd, value);
        Ok(())
    }
    fn execute_c_nop(&self, _: &Processor) -> Result<(), Exception> {
        Ok(())
    }
}

impl Execution for CADDINOP {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        //cnop
        if self.rd(p.state().ir()) == 0 && self.imm(p.state().ir()) == 0 {
            self.execute_c_nop(p)?
        } else {
            self.execute_c_addi(p)?
        }

        let pc = *p.state().pc() + 2;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CIW)]
#[code("0b????????????????000???????????00")]
#[derive(Debug)]
struct CADDI14SPN();

impl Execution for CADDI14SPN {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.imm(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let imm_2: RegT = ((self.imm(p.state().ir()) >> 1) & 0x1) as RegT;
        let imm_3: RegT = (self.imm(p.state().ir()) & 0x1) as RegT;
        let imm_5_4: RegT = ((self.imm(p.state().ir()) >> 6) & 0x3) as RegT;
        let imm_9_6: RegT = ((self.imm(p.state().ir()) >> 2) & 0xf) as RegT;
        let rs1: Wrapping<RegT> = Wrapping(*p.state().xreg(2));
        let rs2: Wrapping<RegT> = Wrapping(imm_2 << 2 | imm_3 << 3 | imm_5_4 << 4 | imm_9_6 << 6);
        let rd = self.rd(p.state().ir());
        let value = (rs1 + rs2).0 & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CI)]
#[code("0b????????????????000???????????10")]
#[derive(Debug)]
struct CSLLI();

impl Execution for CSLLI {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        if self.rd(p.state().ir()) == 0 || self.imm(p.state().ir()) == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        if let Err(_) = p.state().check_xlen(XLen::X64) {
            if self.imm(p.state().ir()) & (1 << 5) != 0 {
                return Err(Exception::IllegalInsn(p.state().ir()));
            }
        }
        let rs1 = *p.state().xreg(self.rs1(p.state().ir()));
        let shamt: RegT = (self.imm(p.state().ir()) as RegT) & ((1 << p.state().config().xlen.len().trailing_zeros()) - 1) as RegT;
        let rd = self.rd(p.state().ir());
        let value = rs1.wrapping_shl(shamt as u32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????100?00????????01")]
#[derive(Debug)]
struct CSRLI();

impl Execution for CSRLI {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let shamt_4_0: RegT = (self.imm(p.state().ir()) & 0x1f) as RegT;
        let shamt_5: RegT = ((self.imm(p.state().ir()) >> 7) & 0x1) as RegT;
        let shamt = shamt_4_0 | shamt_5 << 5;
        if shamt == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        if let Err(_) = p.state().check_xlen(XLen::X64) {
            if shamt_5 != 0 {
                return Err(Exception::IllegalInsn(p.state().ir()));
            }
        }
        let rs1 = *p.state().xreg(self.rs1(p.state().ir()));
        let rd = self.rd(p.state().ir());
        let value = rs1 >> shamt;
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????100?01????????01")]
#[derive(Debug)]
struct CSRAI();

impl Execution for CSRAI {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let shamt_4_0: RegT = (self.imm(p.state().ir()) & 0x1f) as RegT;
        let shamt_5: RegT = ((self.imm(p.state().ir()) >> 7) & 0x1) as RegT;
        let shamt = shamt_4_0 | shamt_5 << 5;
        if shamt == 0 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        if let Err(_) = p.state().check_xlen(XLen::X64) {
            if shamt_5 != 0 {
                return Err(Exception::IllegalInsn(p.state().ir()));
            }
        }
        let rs1 = *p.state().xreg(self.rs1(p.state().ir()));
        let rd = self.rd(p.state().ir());
        let value = sext(rs1.wrapping_shr(shamt as u32), p.state().config().xlen.len() - shamt as usize) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CB)]
#[code("0b????????????????100?10????????01")]
#[derive(Debug)]
struct CANDI();

impl Execution for CANDI {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let imm_4_0: RegT = (self.imm(p.state().ir()) & 0x1f) as RegT;
        let imm_5: RegT = ((self.imm(p.state().ir()) >> 7) & 0x1) as RegT;
        let imm = imm_4_0 | imm_5 << 5;
        let rs1 = *p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = sext(imm, 6) & p.state().config().xlen.mask();
        let rd = self.rd(p.state().ir());
        let value = (rs1 & rs2) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CA)]
#[code("0b????????????????100011???11???01")]
#[derive(Debug)]
struct CAND();

impl Execution for CAND {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let rs1 = *p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = *p.state().xreg(self.rs2(p.state().ir()));
        let rd = self.rd(p.state().ir());
        let value = (rs1 & rs2) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CA)]
#[code("0b????????????????100011???10???01")]
#[derive(Debug)]
struct COR();

impl Execution for COR {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let rs1 = *p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = *p.state().xreg(self.rs2(p.state().ir()));
        let rd = self.rd(p.state().ir());
        let value = (rs1 | rs2) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CA)]
#[code("0b????????????????100011???01???01")]
#[derive(Debug)]
struct CXOR();

impl Execution for CXOR {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let rs1 = *p.state().xreg(self.rs1(p.state().ir()));
        let rs2 = *p.state().xreg(self.rs2(p.state().ir()));
        let rd = self.rd(p.state().ir());
        let value = (rs1 ^ rs2) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CA)]
#[code("0b????????????????100011???00???01")]
#[derive(Debug)]
struct CSUB();

impl Execution for CSUB {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        let rs1: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs2(p.state().ir())));
        let rd = self.rd(p.state().ir());
        let value = (rs1 - rs2).0 & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CA)]
#[code("0b????????????????100111???01???01")]
#[derive(Debug)]
struct CADDW();

impl Execution for CADDW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(*p.state().xreg(self.rs1(p.state().ir())), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(*p.state().xreg(self.rs2(p.state().ir())), 32));
        let rd = self.rd(p.state().ir());
        let value = sext((rs1 + rs2).0, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(CA)]
#[code("0b????????????????100111???00???01")]
#[derive(Debug)]
struct CSUBW();

impl Execution for CSUBW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('c')?;
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(*p.state().xreg(self.rs1(p.state().ir())), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(*p.state().xreg(self.rs2(p.state().ir())), 32));
        let rd = self.rd(p.state().ir());
        let value = sext((rs1 - rs2).0, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 2;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}


