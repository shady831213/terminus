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
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(2));
        let offset_7_6: RegT = self.imm().bit_range(1, 0);
        let offset_5: RegT = self.imm().bit_range(5, 5);
        let offset_4_2: RegT = self.imm().bit_range(4, 2);
        let offset: Wrapping<RegT> = Wrapping(offset_4_2 << 2 | offset_5 << 5 | offset_7_6 << 6);
        let data = p.load_store().load_word((base + offset).0, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
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
        p.state().set_pc(p.state().pc() + 4);
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
        p.state().set_pc(p.state().pc() + 4);
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
        p.state().set_pc(p.state().pc() + 4);
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
        p.state().set_pc(p.state().pc() + 4);
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
        p.state().set_pc(p.state().pc() + 4);
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
        p.state().set_pc(p.state().pc() + 4);
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
        p.state().set_pc(p.state().pc() + 4);
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
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}