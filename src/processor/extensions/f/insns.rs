use crate::prelude::*;
use crate::processor::extensions::f::float::{
    F32Traits, FClass, FCompare, FCompute, FPState, FStore, FToX, FloatInsn, Sign, XToF, F32,
};
use crate::processor::extensions::f::{FLen, FRegT};
use crate::processor::trap::Exception;
use crate::processor::Processor;
use std::cmp::Ordering;
use std::num::Wrapping;

#[derive(Instruction)]
#[format(I)]
#[code("32b?????????????????010?????0000111")]
#[derive(Debug)]
struct FLW();

impl FloatInsn for FLW {}

impl Execution for FLW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let offset: Wrapping<RegT> =
            Wrapping(sext(self.imm(p.state().ir()) as RegT, self.imm_len()));
        let mut data: u32 = 0;
        p.load_store()
            .load_word(p.state(), &(base + offset).0, &mut data, p.mmu())?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(data as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("32b?????????????????010?????0100111")]
#[derive(Debug)]
struct FSW();

impl FloatInsn for FSW {}

impl FStore for FSW {}

impl Execution for FSW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let data = f.freg(self.src(p.state().ir()));
        p.load_store().store_word(
            p.state(),
            &(base + self.offset(p.state().ir())).0,
            unsafe { &*(data as *const FRegT as *const u32) },
            p.mmu(),
        )?;
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0000000??????????????????1010011")]
#[derive(Debug)]
struct FADDS();

impl FloatInsn for FADDS {}

impl FCompute<u32, F32Traits> for FADDS {
    fn opt(&self, ir: &InsnT, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.add(&frs2, Self::rm_from_bits(self.rm(ir)), Some(fp_state))
    }
}

impl Execution for FADDS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, 0)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0000100??????????????????1010011")]
#[derive(Debug)]
struct FSUBS();

impl FloatInsn for FSUBS {}

impl FCompute<u32, F32Traits> for FSUBS {
    fn opt(&self, ir: &InsnT, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.sub(&frs2, Self::rm_from_bits(self.rm(ir)), Some(fp_state))
    }
}

impl Execution for FSUBS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, 0)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0001000??????????????????1010011")]
#[derive(Debug)]
struct FMULS();

impl FloatInsn for FMULS {}

impl FCompute<u32, F32Traits> for FMULS {
    fn opt(&self, ir: &InsnT, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.mul(&frs2, Self::rm_from_bits(self.rm(ir)), Some(fp_state))
    }
}

impl Execution for FMULS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, 0)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0001100??????????????????1010011")]
#[derive(Debug)]
struct FDIVS();

impl FloatInsn for FDIVS {}

impl FCompute<u32, F32Traits> for FDIVS {
    fn opt(&self, ir: &InsnT, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.div(&frs2, Self::rm_from_bits(self.rm(ir)), Some(fp_state))
    }
}

impl Execution for FDIVS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, 0)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b10110000000?????????????1010011")]
#[derive(Debug)]
struct FSQRTS();

impl FloatInsn for FSQRTS {}

impl FCompute<u32, F32Traits> for FSQRTS {
    fn opt(&self, ir: &InsnT, frs1: F32, _: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.sqrt(Self::rm_from_bits(self.rm(ir)), Some(fp_state))
    }
}

impl Execution for FSQRTS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, 0, 0)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0010100??????????000?????1010011")]
#[derive(Debug)]
struct FMINS();

impl FloatInsn for FMINS {}

impl FCompute<u32, F32Traits> for FMINS {
    fn opt(&self, _: &InsnT, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        if frs1.is_nan() && frs2.is_nan() {
            return F32::quiet_nan();
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

impl Execution for FMINS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, 0)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0010100??????????001?????1010011")]
#[derive(Debug)]
struct FMAXS();

impl FloatInsn for FMAXS {}

impl FCompute<u32, F32Traits> for FMAXS {
    fn opt(&self, _: &InsnT, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        if frs1.is_nan() && frs2.is_nan() {
            return F32::quiet_nan();
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

impl Execution for FMAXS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, 0)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b?????00??????????????????1000011")]
#[derive(Debug)]
struct FMADDS();

impl FloatInsn for FMADDS {}

impl FCompute<u32, F32Traits> for FMADDS {
    fn opt(&self, ir: &InsnT, frs1: F32, frs2: F32, frs3: F32, state: &mut FPState) -> F32 {
        frs1.fused_mul_add(&frs2, &frs3, Self::rm_from_bits(self.rm(ir)), Some(state))
    }
}

impl Execution for FMADDS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let rs3: u32 = f.flen.boxed(*f.freg(self.rs3(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, rs3)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b?????00??????????????????1000111")]
#[derive(Debug)]
struct FMSUBS();

impl FloatInsn for FMSUBS {}

impl FCompute<u32, F32Traits> for FMSUBS {
    fn opt(&self, ir: &InsnT, frs1: F32, frs2: F32, frs3: F32, state: &mut FPState) -> F32 {
        frs1.fused_mul_add(
            &frs2,
            &frs3.neg(),
            Self::rm_from_bits(self.rm(ir)),
            Some(state),
        )
    }
}

impl Execution for FMSUBS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let rs3: u32 = f.flen.boxed(*f.freg(self.rs3(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, rs3)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b?????00??????????????????1001011")]
#[derive(Debug)]
struct FMNSUBS();

impl FloatInsn for FMNSUBS {}

impl FCompute<u32, F32Traits> for FMNSUBS {
    fn opt(&self, ir: &InsnT, frs1: F32, frs2: F32, frs3: F32, state: &mut FPState) -> F32 {
        frs1.fused_mul_add(
            &frs2,
            &frs3.neg(),
            Self::rm_from_bits(self.rm(ir)),
            Some(state),
        )
        .neg()
    }
}

impl Execution for FMNSUBS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let rs3: u32 = f.flen.boxed(*f.freg(self.rs3(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, rs3)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b?????00??????????????????1001111")]
#[derive(Debug)]
struct FMNADDS();

impl FloatInsn for FMNADDS {}

impl FCompute<u32, F32Traits> for FMNADDS {
    fn opt(&self, ir: &InsnT, frs1: F32, frs2: F32, frs3: F32, state: &mut FPState) -> F32 {
        frs1.fused_mul_add(&frs2, &frs3, Self::rm_from_bits(self.rm(ir)), Some(state))
            .neg()
    }
}

impl Execution for FMNADDS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let rs3: u32 = f.flen.boxed(*f.freg(self.rs3(p.state().ir())), FLen::F32) as u32;
        let res = self.compute(p.state().ir(), &*f, rs1, rs2, rs3)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b110000000000?????????????1010011")]
#[derive(Debug)]
struct FCVTWS();

impl FloatInsn for FCVTWS {}

impl FToX<u32, F32Traits> for FCVTWS {
    type T = i32;
    fn opt(&self, ir: &InsnT, frs1: F32, state: &mut FPState) -> Self::T {
        if let Some(v) = frs1.to_i32(true, Self::rm_from_bits(self.rm(ir)), Some(state)) {
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

impl Execution for FCVTWS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let res = self.convert(p.state().ir(), &*f, rs1)? as u32;
        let rd = self.rd(p.state().ir());
        let value = sext(res as RegT, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b110000000001?????????????1010011")]
#[derive(Debug)]
struct FCVTWUS();

impl FloatInsn for FCVTWUS {}

impl FToX<u32, F32Traits> for FCVTWUS {
    type T = u32;
    fn opt(&self, ir: &InsnT, frs1: F32, state: &mut FPState) -> Self::T {
        if let Some(v) = frs1.to_u32(true, Self::rm_from_bits(self.rm(ir)), Some(state)) {
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

impl Execution for FCVTWUS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let res = self.convert(p.state().ir(), &*f, rs1)?;
        let rd = self.rd(p.state().ir());
        let value = sext(res as RegT, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b110000000010?????????????1010011")]
#[derive(Debug)]
struct FCVTLS();

impl FloatInsn for FCVTLS {}

impl FToX<u32, F32Traits> for FCVTLS {
    type T = i64;
    fn opt(&self, ir: &InsnT, frs1: F32, state: &mut FPState) -> Self::T {
        if let Some(v) = frs1.to_i64(true, Self::rm_from_bits(self.rm(ir)), Some(state)) {
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

impl Execution for FCVTLS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let res = self.convert(p.state().ir(), &*f, rs1)? as u64;
        let rd = self.rd(p.state().ir());
        let value = res as RegT & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b110000000011?????????????1010011")]
#[derive(Debug)]
struct FCVTLUS();

impl FloatInsn for FCVTLUS {}

impl FToX<u32, F32Traits> for FCVTLUS {
    type T = u64;
    fn opt(&self, ir: &InsnT, frs1: F32, state: &mut FPState) -> Self::T {
        if let Some(v) = frs1.to_u64(true, Self::rm_from_bits(self.rm(ir)), Some(state)) {
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

impl Execution for FCVTLUS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let res = self.convert(p.state().ir(), &*f, rs1)?;
        let rd = self.rd(p.state().ir());
        let value = res as RegT & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b110100000000?????????????1010011")]
#[derive(Debug)]
struct FCVTSW();

impl FloatInsn for FCVTSW {}

impl XToF<u32, F32Traits> for FCVTSW {
    type T = i32;
    fn opt(&self, ir: &InsnT, rs1: Self::T, state: &mut FPState) -> F32 {
        F32::from_i32(rs1, Self::rm_from_bits(self.rm(ir)), Some(state))
    }
}

impl Execution for FCVTSW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: RegT = sext(*p.state().xreg(self.rs1(p.state().ir())), 32);
        let fres = self.convert(p.state().ir(), &*f, rs1 as i32)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(fres as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b110100000001?????????????1010011")]
#[derive(Debug)]
struct FCVTSWU();

impl FloatInsn for FCVTSWU {}

impl XToF<u32, F32Traits> for FCVTSWU {
    type T = u32;
    fn opt(&self, ir: &InsnT, rs1: Self::T, state: &mut FPState) -> F32 {
        F32::from_u32(rs1, Self::rm_from_bits(self.rm(ir)), Some(state))
    }
}

impl Execution for FCVTSWU {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: RegT = *p.state().xreg(self.rs1(p.state().ir())) & 0xffff_ffff;
        let fres = self.convert(p.state().ir(), &*f, rs1 as u32)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(fres as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b110100000010?????????????1010011")]
#[derive(Debug)]
struct FCVTSL();

impl FloatInsn for FCVTSL {}

impl XToF<u32, F32Traits> for FCVTSL {
    type T = i64;
    fn opt(&self, ir: &InsnT, rs1: Self::T, state: &mut FPState) -> F32 {
        F32::from_i64(rs1, Self::rm_from_bits(self.rm(ir)), Some(state))
    }
}

impl Execution for FCVTSL {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = *p.state().xreg(self.rs1(p.state().ir()));
        let fres = self.convert(p.state().ir(), &*f, rs1 as i64)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(fres as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b110100000011?????????????1010011")]
#[derive(Debug)]
struct FCVTSLU();

impl FloatInsn for FCVTSLU {}

impl XToF<u32, F32Traits> for FCVTSLU {
    type T = u64;
    fn opt(&self, ir: &InsnT, rs1: Self::T, state: &mut FPState) -> F32 {
        F32::from_u64(rs1, Self::rm_from_bits(self.rm(ir)), Some(state))
    }
}

impl Execution for FCVTSLU {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = *p.state().xreg(self.rs1(p.state().ir()));
        let fres = self.convert(p.state().ir(), &*f, rs1 as u64)?;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(fres as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0010000??????????000?????1010011")]
#[derive(Debug)]
struct FSGNJS();

impl FloatInsn for FSGNJS {}

impl Execution for FSGNJS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = rs1 & ((1 << 31) - 1) | rs2 & (1 << 31);
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0010000??????????001?????1010011")]
#[derive(Debug)]
struct FSGNJNS();

impl FloatInsn for FSGNJNS {}

impl Execution for FSGNJNS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = rs1 & ((1 << 31) - 1) | !rs2 & (1 << 31);
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b0010000??????????010?????1010011")]
#[derive(Debug)]
struct FSGNJXS();

impl FloatInsn for FSGNJXS {}

impl Execution for FSGNJXS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = rs1 & ((1 << 31) - 1) | (rs1 ^ rs2) & (1 << 31);
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(res as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b1010000??????????010?????1010011")]
#[derive(Debug)]
struct FEQS();

impl FloatInsn for FEQS {}

impl FCompare<u32, F32Traits> for FEQS {}

impl Execution for FEQS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        if let Some(Ordering::Equal) = self.compare(p.state().ir(), &*f, rs1, rs2, false)? {
            let rd = self.rd(p.state().ir());
            let value = 1;
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = 0;
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b1010000??????????001?????1010011")]
#[derive(Debug)]
struct FLTS();

impl FloatInsn for FLTS {}

impl FCompare<u32, F32Traits> for FLTS {}

impl Execution for FLTS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        if let Some(Ordering::Less) = self.compare(p.state().ir(), &*f, rs1, rs2, true)? {
            let rd = self.rd(p.state().ir());
            let value = 1;
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = 0;
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b1010000??????????000?????1010011")]
#[derive(Debug)]
struct FLES();

impl FloatInsn for FLES {}

impl FCompare<u32, F32Traits> for FLES {}

impl Execution for FLES {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rs2: u32 = f.flen.boxed(*f.freg(self.rs2(p.state().ir())), FLen::F32) as u32;
        let res = self.compare(p.state().ir(), &*f, rs1, rs2, true)?;
        if let Some(Ordering::Equal) = res {
            let rd = self.rd(p.state().ir());
            let value = 1;
            p.state_mut().set_xreg(rd, value);
        } else if let Some(Ordering::Less) = res {
            let rd = self.rd(p.state().ir());
            let value = 1;
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = 0;
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b111000000000?????001?????1010011")]
#[derive(Debug)]
struct FCLASSS();

impl FloatInsn for FCLASSS {}

impl FClass<u32, F32Traits> for FCLASSS {}

impl Execution for FCLASSS {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.flen.boxed(*f.freg(self.rs1(p.state().ir())), FLen::F32) as u32;
        let rd = self.rd(p.state().ir());
        let value = self.class(rs1);
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b111000000000?????000?????1010011")]
#[derive(Debug)]
struct FMVXW();

impl FloatInsn for FMVXW {}

impl Execution for FMVXW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let data: RegT = (*f.freg(self.rs1(p.state().ir())) & 0xffff_ffff) as RegT;
        let rd = self.rd(p.state().ir());
        let value = sext(data, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("32b111100000000?????000?????1010011")]
#[derive(Debug)]
struct FMVWX();

impl FloatInsn for FMVWX {}

impl Execution for FMVWX {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let data: RegT = *p.state().xreg(self.rs1(p.state().ir())) & 0xffff_ffff;
        let pc = *p.state().pc() + 4;
        let rd = self.rd(p.state().ir());
        let value = f.flen.padding(data as FRegT, FLen::F32);
        self.get_f_ext_mut(p).set_freg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}
