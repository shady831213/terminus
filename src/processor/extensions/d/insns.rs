use crate::processor::insn_define::*;
use std::num::Wrapping;
use crate::processor::extensions::f::{FRegT, FLen};
use crate::processor::extensions::f::float::*;
use std::cmp::Ordering;

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
        f.set_freg(self.rd() as RegT, f.flen.padding(data as FRegT, FLen::F64));
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
        let data = f.freg(self.src()) as u64;
        p.load_store().store_double_word((base + self.offset()).0, data as RegT, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????????????1010011")]
#[derive(Debug)]
struct FADDD(InsnT);

impl FloatInsn for FADDD {}

impl FCompute<u64, F64Traits> for FADDD {
    fn opt(&self, frs1: F64, frs2: F64, _: F64, fp_state: &mut FPState) -> F64 {
        frs1.add(&frs2, Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FADDD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000101??????????????????1010011")]
#[derive(Debug)]
struct FSUBD(InsnT);

impl FloatInsn for FSUBD {}

impl FCompute<u64, F64Traits> for FSUBD {
    fn opt(&self, frs1: F64, frs2: F64, _: F64, fp_state: &mut FPState) -> F64 {
        frs1.sub(&frs2, Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FSUBD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0001001??????????????????1010011")]
#[derive(Debug)]
struct FMULD(InsnT);

impl FloatInsn for FMULD {}

impl FCompute<u64, F64Traits> for FMULD {
    fn opt(&self, frs1: F64, frs2: F64, _: F64, fp_state: &mut FPState) -> F64 {
        frs1.mul(&frs2, Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FMULD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0001101??????????????????1010011")]
#[derive(Debug)]
struct FDIVD(InsnT);

impl FloatInsn for FDIVD {}

impl FCompute<u64, F64Traits> for FDIVD {
    fn opt(&self, frs1: F64, frs2: F64, _: F64, fp_state: &mut FPState) -> F64 {
        frs1.div(&frs2, Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FDIVD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b010110100000?????????????1010011")]
#[derive(Debug)]
struct FSQRTD(InsnT);

impl FloatInsn for FSQRTD {}

impl FCompute<u64, F64Traits> for FSQRTD {
    fn opt(&self, frs1: F64, _: F64, _: F64, fp_state: &mut FPState) -> F64 {
        frs1.sqrt(Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FSQRTD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, 0, 0)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010101??????????000?????1010011")]
#[derive(Debug)]
struct FMIND(InsnT);

impl FloatInsn for FMIND {}

impl FCompute<u64, F64Traits> for FMIND {
    fn opt(&self, frs1: F64, frs2: F64, _: F64, fp_state: &mut FPState) -> F64 {
        if frs1.is_nan() && frs2.is_nan() {
            return F64::quiet_nan();
        }
        if frs1.is_negative_zero() && frs2.is_zero() {
            return frs1;
        }
        if let Some(Ordering::Less) = frs1.compare_quiet(&frs2, Some(fp_state)) {
            frs1
        } else {
            frs2
        }
    }
}

impl Execution for FMIND {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010101??????????001?????1010011")]
#[derive(Debug)]
struct FMAXD(InsnT);

impl FloatInsn for FMAXD {}

impl FCompute<u64, F64Traits> for FMAXD {
    fn opt(&self, frs1: F64, frs2: F64, _: F64, fp_state: &mut FPState) -> F64 {
        if frs1.is_nan() && frs2.is_nan() {
            return F64::quiet_nan();
        }
        if frs1.is_positive_zero() && frs2.is_zero() {
            return frs1;
        }
        if let Some(Ordering::Greater) = frs1.compare_quiet(&frs2, Some(fp_state)) {
            frs1
        } else {
            frs2
        }
    }
}

impl Execution for FMAXD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b?????01??????????????????1000011")]
#[derive(Debug)]
struct FMADDD(InsnT);

impl FloatInsn for FMADDD {}

impl FCompute<u64, F64Traits> for FMADDD {
    fn opt(&self, frs1: F64, frs2: F64, frs3: F64, state: &mut FPState) -> F64 {
        frs1.fused_mul_add(&frs2, &frs3, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FMADDD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let rs3: u64 = f.freg(self.rs3() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, rs3)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b?????01??????????????????1000111")]
#[derive(Debug)]
struct FMSUBD(InsnT);

impl FloatInsn for FMSUBD {}

impl FCompute<u64, F64Traits> for FMSUBD {
    fn opt(&self, frs1: F64, frs2: F64, frs3: F64, state: &mut FPState) -> F64 {
        frs1.fused_mul_add(&frs2, &frs3.neg(), Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FMSUBD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let rs3: u64 = f.freg(self.rs3() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, rs3)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(R)]
#[code("0b?????01??????????????????1001011")]
#[derive(Debug)]
struct FMNSUBD(InsnT);

impl FloatInsn for FMNSUBD {}

impl FCompute<u64, F64Traits> for FMNSUBD {
    fn opt(&self, frs1: F64, frs2: F64, frs3: F64, state: &mut FPState) -> F64 {
        frs1.fused_mul_add(&frs2, &frs3.neg(), Self::rm_from_bits(self.rm()), Some(state)).neg()
    }
}

impl Execution for FMNSUBD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let rs3: u64 = f.freg(self.rs3() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, rs3)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b?????01??????????????????1001111")]
#[derive(Debug)]
struct FMNADDD(InsnT);

impl FloatInsn for FMNADDD {}

impl FCompute<u64, F64Traits> for FMNADDD {
    fn opt(&self, frs1: F64, frs2: F64, frs3: F64, state: &mut FPState) -> F64 {
        frs1.fused_mul_add(&frs2, &frs3, Self::rm_from_bits(self.rm()), Some(state)).neg()
    }
}

impl Execution for FMNADDD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let rs3: u64 = f.freg(self.rs3() as RegT).bit_range(63, 0);
        let res = self.compute(f.deref(), rs1, rs2, rs3)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(R)]
#[code("0b110000100000?????????????1010011")]
#[derive(Debug)]
struct FCVTWD(InsnT);

impl FloatInsn for FCVTWD {}

impl FToX<u64, F64Traits> for FCVTWD {
    type T = i32;
    fn opt(&self, frs1: F64, state: &mut FPState) -> Self::T {
        if let Some(v) = frs1.to_i32(true, Self::rm_from_bits(self.rm()), Some(state)) {
            v
        } else {
            if frs1.is_nan() || frs1.sign() == Sign::Positive {
                ((1u32 << 31) - 1) as Self::T
            } else {
                (1u32 << 31) as Self::T
            }
        }
    }
}

impl Execution for FCVTWD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let res = self.convert(f.deref(), rs1)? as u32;
        p.state().set_xreg(self.rd() as RegT, sext(res as RegT, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110000100001?????????????1010011")]
#[derive(Debug)]
struct FCVTWUD(InsnT);

impl FloatInsn for FCVTWUD {}

impl FToX<u64, F64Traits> for FCVTWUD {
    type T = u32;
    fn opt(&self, frs1: F64, state: &mut FPState) -> Self::T {
        if let Some(v) = frs1.to_u32(true, Self::rm_from_bits(self.rm()), Some(state)) {
            v
        } else {
            if frs1.is_nan() || frs1.sign() == Sign::Positive {
                -1i32 as Self::T
            } else {
                0
            }
        }
    }
}

impl Execution for FCVTWUD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let res = self.convert(f.deref(), rs1)?;
        p.state().set_xreg(self.rd() as RegT, sext(res as RegT, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110000100010?????????????1010011")]
#[derive(Debug)]
struct FCVTLD(InsnT);

impl FloatInsn for FCVTLD {}

impl FToX<u64, F64Traits> for FCVTLD {
    type T = i64;
    fn opt(&self, frs1: F64, state: &mut FPState) -> Self::T {
        if let Some(v) = frs1.to_i64(true, Self::rm_from_bits(self.rm()), Some(state)) {
            v
        } else {
            if frs1.is_nan() || frs1.sign() == Sign::Positive {
                ((1u64 << 63) - 1) as Self::T
            } else {
                (1u64 << 63) as Self::T
            }
        }
    }
}

impl Execution for FCVTLD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let res = self.convert(f.deref(), rs1)? as u64;
        p.state().set_xreg(self.rd() as RegT, res as RegT & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110000100011?????????????1010011")]
#[derive(Debug)]
struct FCVTLUD(InsnT);

impl FloatInsn for FCVTLUD {}

impl FToX<u64, F64Traits> for FCVTLUD {
    type T = u64;
    fn opt(&self, frs1: F64, state: &mut FPState) -> Self::T {
        if let Some(v) = frs1.to_u64(true, Self::rm_from_bits(self.rm()), Some(state)) {
            v
        } else {
            if frs1.is_nan() || frs1.sign() == Sign::Positive {
                -1i64 as Self::T
            } else {
                0
            }
        }
    }
}

impl Execution for FCVTLUD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let res = self.convert(f.deref(), rs1)?;
        p.state().set_xreg(self.rd() as RegT, res as RegT & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100100000?????????????1010011")]
#[derive(Debug)]
struct FCVTDW(InsnT);

impl FloatInsn for FCVTDW {}

impl XToF<u64, F64Traits> for FCVTDW {
    type T = i32;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F64 {
        F64::from_i32(rs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTDW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = sext(p.state().xreg(self.rs1() as RegT), 32);
        let fres = self.convert(f.deref(), rs1 as i32)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(fres as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100100001?????????????1010011")]
#[derive(Debug)]
struct FCVTDWU(InsnT);

impl FloatInsn for FCVTDWU {}

impl XToF<u64, F64Traits> for FCVTDWU {
    type T = u32;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F64 {
        F64::from_u32(rs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTDWU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1 as u32)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(fres as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100100010?????????????1010011")]
#[derive(Debug)]
struct FCVTDL(InsnT);

impl FloatInsn for FCVTDL {}

impl XToF<u64, F64Traits> for FCVTDL {
    type T = i64;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F64 {
        F64::from_i64(rs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTDL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT);
        let fres = self.convert(f.deref(), rs1 as i64)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(fres as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100100011?????????????1010011")]
#[derive(Debug)]
struct FCVTDLU(InsnT);

impl FloatInsn for FCVTDLU {}

impl XToF<u64, F64Traits> for FCVTDLU {
    type T = u64;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F64 {
        F64::from_u64(rs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTDLU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT);
        let fres = self.convert(f.deref(), rs1 as u64)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(fres as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b010000000001?????????????1010011")]
#[derive(Debug)]
struct FCVTSD(InsnT);

impl FloatInsn for FCVTSD {}

impl FToX<u64, F64Traits> for FCVTSD {
    type T = u32;
    fn opt(&self, frs1: F64, state: &mut FPState) -> Self::T {
        *frs1.convert_to_float::<F32Traits>(Self::rm_from_bits(self.rm()), Some(state)).bits()
    }
}

impl Execution for FCVTSD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let fres = self.convert(f.deref(), rs1)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(fres as FRegT, FLen::F32));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b010000100000?????????????1010011")]
#[derive(Debug)]
struct FCVTDS(InsnT);

impl FloatInsn for FCVTDS {}

impl XToF<u64, F64Traits> for FCVTDS {
    type T = u32;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F64 {
        let frs1 = F32::from_bits(rs1);
        F64::convert_from_float::<F32Traits>(&frs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTDS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1)?;
        f.set_freg(self.rd() as RegT, f.flen.padding(fres as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010001??????????000?????1010011")]
#[derive(Debug)]
struct FSGNJD(InsnT);

impl FloatInsn for FSGNJD {}

impl Execution for FSGNJD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = rs1 & ((1 << 63) - 1) | rs2 & (1 << 63);
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010001??????????001?????1010011")]
#[derive(Debug)]
struct FSGNJND(InsnT);

impl FloatInsn for FSGNJND {}

impl Execution for FSGNJND {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = rs1 & ((1 << 63) - 1) | !rs2 & (1 << 63);
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010001??????????010?????1010011")]
#[derive(Debug)]
struct FSGNJXD(InsnT);

impl FloatInsn for FSGNJXD {}

impl Execution for FSGNJXD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = rs1 & ((1 << 63) - 1) | (rs1 ^ rs2) & (1 << 63);
        f.set_freg(self.rd() as RegT, f.flen.padding(res as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b1010001??????????010?????1010011")]
#[derive(Debug)]
struct FEQD(InsnT);

impl FloatInsn for FEQD {}

impl FCompare<u64, F64Traits> for FEQD {}

impl Execution for FEQD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        if let Some(Ordering::Equal) = self.compare(f.deref(), rs1, rs2, false)? {
            p.state().set_xreg(self.rd() as RegT, 1);
        } else {
            p.state().set_xreg(self.rd() as RegT, 0);
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b1010001??????????001?????1010011")]
#[derive(Debug)]
struct FLTD(InsnT);

impl FloatInsn for FLTD {}

impl FCompare<u64, F64Traits> for FLTD {}

impl Execution for FLTD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        if let Some(Ordering::Less) = self.compare(f.deref(), rs1, rs2, true)? {
            p.state().set_xreg(self.rd() as RegT, 1);
        } else {
            p.state().set_xreg(self.rd() as RegT, 0);
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b1010001??????????000?????1010011")]
#[derive(Debug)]
struct FLED(InsnT);

impl FloatInsn for FLED {}

impl FCompare<u64, F64Traits> for FLED {}

impl Execution for FLED {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        let rs2: u64 = f.freg(self.rs2() as RegT).bit_range(63, 0);
        let res = self.compare(f.deref(), rs1, rs2, true)?;
        if let Some(Ordering::Equal) = res {
            p.state().set_xreg(self.rd() as RegT, 1);
        } else if let Some(Ordering::Less) = res {
            p.state().set_xreg(self.rd() as RegT, 1);
        } else {
            p.state().set_xreg(self.rd() as RegT, 0);
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(R)]
#[code("0b111000100000?????001?????1010011")]
#[derive(Debug)]
struct FCLASSD(InsnT);

impl FloatInsn for FCLASSD {}

impl FClass<u64, F64Traits> for FCLASSD {}

impl Execution for FCLASSD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let rs1: u64 = f.freg(self.rs1() as RegT).bit_range(63, 0);
        p.state().set_xreg(self.rd() as RegT, self.class(rs1));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b111000100000?????000?????1010011")]
#[derive(Debug)]
struct FMVXD(InsnT);

impl FloatInsn for FMVXD {}

impl Execution for FMVXD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let data: RegT = f.freg(self.rs1() as RegT).bit_range(63, 0);
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b111100100000?????000?????1010011")]
#[derive(Debug)]
struct FMVDX(InsnT);

impl FloatInsn for FMVDX {}

impl Execution for FMVDX {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('d')?;
        let f = self.get_f_ext(p)?;
        let data: RegT = p.state().xreg(self.rs1() as RegT).bit_range(63, 0);
        f.set_freg(self.rd() as RegT, f.flen.padding(data as FRegT, FLen::F64));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}