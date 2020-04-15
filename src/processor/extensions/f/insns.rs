use crate::processor::insn_define::*;
use std::num::Wrapping;
use crate::processor::extensions::f::{ExtensionF, FRegT};
use crate::processor::extensions::Extension;
use std::rc::Rc;
use simple_soft_float::{F32, FPState, RoundingMode, Sign, StatusFlags, FloatClass, FloatTraits, Float, F32Traits, FloatBitsType};
use std::cmp::Ordering;

trait FloatInsn: InstructionImp {
    fn get_f_ext(&self, p: &Processor) -> Result<Rc<ExtensionF>, Exception> {
        p.state().check_extension('f')?;
        if let Some(Extension::F(f)) = p.state().extensions().get(&'f') {
            if f.dirty() == 0 {
                Err(Exception::IllegalInsn(self.ir()))
            } else {
                Ok(f.clone())
            }
        } else {
            Err(Exception::IllegalInsn(self.ir()))
        }
    }
    fn rm(&self) -> RegT {
        self.ir().bit_range(14, 12)
    }

    fn rs3(&self) -> InsnT {
        self.ir().bit_range(31, 27)
    }

    fn rm_from_bits(bits: RegT) -> Option<RoundingMode> {
        match bits {
            0 => Some(RoundingMode::TiesToEven),
            1 => Some(RoundingMode::TowardZero),
            2 => Some(RoundingMode::TowardNegative),
            3 => Some(RoundingMode::TowardPositive),
            4 => Some(RoundingMode::TiesToAway),
            _ => None
        }
    }

    fn status_flags_to_bits(s: &StatusFlags) -> RegT {
        (s.bits() << 27).reverse_bits() as RegT
    }
}


trait FStore: FloatInsn {
    fn offset(&self) -> Wrapping<RegT> {
        let high: RegT = self.imm().bit_range(11, 5);
        let low = self.rd() as RegT;
        Wrapping(sext(high << 5 | low, self.imm_len()))
    }
    fn src(&self) -> RegT {
        self.imm().bit_range(4, 0)
    }
}

trait FCompute<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    fn opt(&self, frs1: Float<FpTrait>, frs2: Float<FpTrait>, frs3: Float<FpTrait>, state: &mut FPState) -> Float<FpTrait>;
    fn compute(&self, f: &ExtensionF, rs1: Bits, rs2: Bits, rs3: Bits) -> Result<Bits, Exception> {
        let mut fp_state = FPState::default();
        fp_state.rounding_mode = if let Some(rm) = Self::rm_from_bits(f.csrs.frm().get()) {
            rm
        } else {
            if self.rm() == 7 { return Err(Exception::IllegalInsn(self.ir())); } else { RoundingMode::default() }
        };
        let frs1 = Float::<FpTrait>::from_bits(rs1);
        let frs2 = Float::<FpTrait>::from_bits(rs2);
        let frs3 = Float::<FpTrait>::from_bits(rs3);
        let fres = self.opt(frs1, frs2, frs3, &mut fp_state);
        f.csrs.fflags_mut().set(Self::status_flags_to_bits(&fp_state.status_flags));
        Ok(*fres.bits())
    }
}


trait FToInt<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    type T;
    fn opt(&self, frs1: Float<FpTrait>, state: &mut FPState) -> Self::T;
    fn convert(&self, f: &ExtensionF, rs1: Bits) -> Result<Self::T, Exception> {
        let mut fp_state = FPState::default();
        fp_state.rounding_mode = if let Some(rm) = Self::rm_from_bits(f.csrs.frm().get()) {
            rm
        } else {
            if self.rm() == 7 { return Err(Exception::IllegalInsn(self.ir())); } else { RoundingMode::default() }
        };
        let fres = Float::<FpTrait>::from_bits(rs1);
        let res: Self::T = self.opt(fres, &mut fp_state);
        f.csrs.fflags_mut().set(Self::status_flags_to_bits(&fp_state.status_flags));
        Ok(res)
    }
}

trait IntToF<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    type T;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> Float<FpTrait>;
    fn convert(&self, f: &ExtensionF, rs1: Self::T) -> Result<Bits, Exception> {
        let mut fp_state = FPState::default();
        fp_state.rounding_mode = if let Some(rm) = Self::rm_from_bits(f.csrs.frm().get()) {
            rm
        } else {
            if self.rm() == 7 { return Err(Exception::IllegalInsn(self.ir())); } else { RoundingMode::default() }
        };
        let res = self.opt(rs1, &mut fp_state);
        f.csrs.fflags_mut().set(Self::status_flags_to_bits(&fp_state.status_flags));
        Ok(*res.bits())
    }
}

trait FCompare<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    fn compare(&self, f: &ExtensionF, rs1: Bits, rs2: Bits, check_nan: bool) -> Result<Option<Ordering>, Exception> {
        let mut fp_state = FPState::default();
        fp_state.rounding_mode = if let Some(rm) = Self::rm_from_bits(f.csrs.frm().get()) {
            rm
        } else {
            if self.rm() == 7 { return Err(Exception::IllegalInsn(self.ir())); } else { RoundingMode::default() }
        };
        let frs1 =  Float::<FpTrait>::from_bits(rs1);
        let frs2 = Float::<FpTrait>::from_bits(rs2);
        let res = frs1.compare_quiet(&frs2, Some(&mut fp_state));
        if check_nan {
            if frs1.is_nan() || frs2.is_nan() {
                fp_state.status_flags = StatusFlags::INVALID_OPERATION;
            }
        }
        f.csrs.fflags_mut().set(Self::status_flags_to_bits(&fp_state.status_flags));
        Ok(res)
    }
}

trait FClass<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    fn class(&self, rs1: Bits) -> RegT {
        let frs1 = Float::<FpTrait>::from_bits(rs1);
        1 << match frs1.class() {
            FloatClass::NegativeInfinity => 0,
            FloatClass::NegativeNormal => 1,
            FloatClass::NegativeSubnormal => 2,
            FloatClass::NegativeZero => 3,
            FloatClass::PositiveZero => 4,
            FloatClass::PositiveSubnormal => 5,
            FloatClass::PositiveNormal => 6,
            FloatClass::PositiveInfinity => 7,
            FloatClass::SignalingNaN => 8,
            FloatClass::QuietNaN => 9
        }
    }
}


#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0000111")]
#[derive(Debug)]
struct FLW(InsnT);

impl FloatInsn for FLW {}

impl Execution for FLW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_word((base + offset).0, p.mmu())?;
        f.set_freg(self.rd() as RegT, data as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0100111")]
#[derive(Debug)]
struct FSW(InsnT);

impl FloatInsn for FSW {}

impl FStore for FSW {}

impl Execution for FSW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let data = f.freg(self.src());
        p.load_store().store_word((base + self.offset()).0, data as RegT, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????????????1010011")]
#[derive(Debug)]
struct FADDS(InsnT);

impl FloatInsn for FADDS {}

impl FCompute<u32, F32Traits> for FADDS {
    fn opt(&self, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.add(&frs2, Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FADDS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000100??????????????????1010011")]
#[derive(Debug)]
struct FSUBS(InsnT);

impl FloatInsn for FSUBS {}

impl FCompute<u32, F32Traits> for FSUBS {
    fn opt(&self, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.sub(&frs2, Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FSUBS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0001000??????????????????1010011")]
#[derive(Debug)]
struct FMULS(InsnT);

impl FloatInsn for FMULS {}

impl FCompute<u32, F32Traits> for FMULS {
    fn opt(&self, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.mul(&frs2, Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FMULS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0001100??????????????????1010011")]
#[derive(Debug)]
struct FDIVS(InsnT);

impl FloatInsn for FDIVS {}

impl FCompute<u32, F32Traits> for FDIVS {
    fn opt(&self, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.div(&frs2, Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FDIVS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b10110000000?????????????1010011")]
#[derive(Debug)]
struct FSQRTS(InsnT);

impl FloatInsn for FSQRTS {}

impl FCompute<u32, F32Traits> for FSQRTS {
    fn opt(&self, frs1: F32, _: F32, _: F32, fp_state: &mut FPState) -> F32 {
        frs1.sqrt(Self::rm_from_bits(self.rm()), Some(fp_state))
    }
}

impl Execution for FSQRTS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, 0, 0)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010100??????????000?????1010011")]
#[derive(Debug)]
struct FMINS(InsnT);

impl FloatInsn for FMINS {}

impl FCompute<u32, F32Traits> for FMINS {
    fn opt(&self, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
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
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010100??????????001?????1010011")]
#[derive(Debug)]
struct FMAXS(InsnT);

impl FloatInsn for FMAXS {}

impl FCompute<u32, F32Traits> for FMAXS {
    fn opt(&self, frs1: F32, frs2: F32, _: F32, fp_state: &mut FPState) -> F32 {
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
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, 0)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b?????00??????????????????1000011")]
#[derive(Debug)]
struct FMADDS(InsnT);

impl FloatInsn for FMADDS {}

impl FCompute<u32, F32Traits> for FMADDS {
    fn opt(&self, frs1: F32, frs2: F32, frs3: F32, state: &mut FPState) -> F32 {
        frs1.fused_mul_add(&frs2, &frs3, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FMADDS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let rs3: u32 = f.freg(self.rs3() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, rs3)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b?????00??????????????????1000111")]
#[derive(Debug)]
struct FMSUBS(InsnT);

impl FloatInsn for FMSUBS {}

impl FCompute<u32, F32Traits> for FMSUBS {
    fn opt(&self, frs1: F32, frs2: F32, frs3: F32, state: &mut FPState) -> F32 {
        frs1.fused_mul_add(&frs2, &frs3.neg(), Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FMSUBS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let rs3: u32 = f.freg(self.rs3() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, rs3)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(R)]
#[code("0b?????00??????????????????1001011")]
#[derive(Debug)]
struct FMNSUBS(InsnT);

impl FloatInsn for FMNSUBS {}

impl FCompute<u32, F32Traits> for FMNSUBS {
    fn opt(&self, frs1: F32, frs2: F32, frs3: F32, state: &mut FPState) -> F32 {
        frs1.fused_mul_add(&frs2, &frs3.neg(), Self::rm_from_bits(self.rm()), Some(state)).neg()
    }
}

impl Execution for FMNSUBS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let rs3: u32 = f.freg(self.rs3() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, rs3)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b?????00??????????????????1001111")]
#[derive(Debug)]
struct FMNADDS(InsnT);

impl FloatInsn for FMNADDS {}

impl FCompute<u32, F32Traits> for FMNADDS {
    fn opt(&self, frs1: F32, frs2: F32, frs3: F32, state: &mut FPState) -> F32 {
        frs1.fused_mul_add(&frs2, &frs3, Self::rm_from_bits(self.rm()), Some(state)).neg()
    }
}

impl Execution for FMNADDS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let rs3: u32 = f.freg(self.rs3() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2, rs3)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(R)]
#[code("0b110000000000?????????????1010011")]
#[derive(Debug)]
struct FCVTWS(InsnT);

impl FloatInsn for FCVTWS {}

impl FToInt<u32, F32Traits> for FCVTWS {
    type T = i32;
    fn opt(&self, frs1: F32, state: &mut FPState) -> Self::T {
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

impl Execution for FCVTWS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let res = self.convert(f.deref(), rs1)? as u32;
        p.state().set_xreg(self.rd() as RegT, sext(res as RegT, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110000000001?????????????1010011")]
#[derive(Debug)]
struct FCVTWUS(InsnT);

impl FloatInsn for FCVTWUS {}

impl FToInt<u32, F32Traits> for FCVTWUS {
    type T = u32;
    fn opt(&self, frs1: F32, state: &mut FPState) -> Self::T {
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

impl Execution for FCVTWUS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let res = self.convert(f.deref(), rs1)?;
        p.state().set_xreg(self.rd() as RegT, sext(res as RegT, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110000000010?????????????1010011")]
#[derive(Debug)]
struct FCVTLS(InsnT);

impl FloatInsn for FCVTLS {}

impl FToInt<u32, F32Traits> for FCVTLS {
    type T = i64;
    fn opt(&self, frs1: F32, state: &mut FPState) -> Self::T {
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

impl Execution for FCVTLS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let res = self.convert(f.deref(), rs1)? as u64;
        p.state().set_xreg(self.rd() as RegT, res as RegT & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110000000011?????????????1010011")]
#[derive(Debug)]
struct FCVTLUS(InsnT);

impl FloatInsn for FCVTLUS {}

impl FToInt<u32, F32Traits> for FCVTLUS {
    type T = u64;
    fn opt(&self, frs1: F32, state: &mut FPState) -> Self::T {
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

impl Execution for FCVTLUS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let res = self.convert(f.deref(), rs1)?;
        p.state().set_xreg(self.rd() as RegT, res as RegT & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000000?????????????1010011")]
#[derive(Debug)]
struct FCVTSW(InsnT);

impl FloatInsn for FCVTSW {}

impl IntToF<u32, F32Traits> for FCVTSW {
    type T = i32;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F32 {
        F32::from_i32(rs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTSW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: RegT = sext(p.state().xreg(self.rs1() as RegT), 32);
        let fres = self.convert(f.deref(), rs1 as i32)?;
        f.set_freg(self.rd() as RegT, fres as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000001?????????????1010011")]
#[derive(Debug)]
struct FCVTSWU(InsnT);

impl FloatInsn for FCVTSWU {}

impl IntToF<u32, F32Traits> for FCVTSWU {
    type T = u32;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F32 {
        F32::from_u32(rs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTSWU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1 as u32)?;
        f.set_freg(self.rd() as RegT, fres as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000010?????????????1010011")]
#[derive(Debug)]
struct FCVTSL(InsnT);

impl FloatInsn for FCVTSL {}

impl IntToF<u32, F32Traits> for FCVTSL {
    type T = i64;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F32 {
        F32::from_i64(rs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTSL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT);
        let fres = self.convert(f.deref(), rs1 as i64)?;
        f.set_freg(self.rd() as RegT, fres as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000011?????????????1010011")]
#[derive(Debug)]
struct FCVTSLU(InsnT);

impl FloatInsn for FCVTSLU {}

impl IntToF<u32, F32Traits> for FCVTSLU {
    type T = u64;
    fn opt(&self, rs1: Self::T, state: &mut FPState) -> F32 {
        F32::from_u64(rs1, Self::rm_from_bits(self.rm()), Some(state))
    }
}

impl Execution for FCVTSLU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT);
        let fres = self.convert(f.deref(), rs1 as u64)?;
        f.set_freg(self.rd() as RegT, fres as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010000??????????000?????1010011")]
#[derive(Debug)]
struct FSGNJS(InsnT);

impl FloatInsn for FSGNJS {}

impl Execution for FSGNJS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = rs1 & ((1 << 31) - 1) | rs2 & (1 << 31);
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010000??????????001?????1010011")]
#[derive(Debug)]
struct FSGNJNS(InsnT);

impl FloatInsn for FSGNJNS {}

impl Execution for FSGNJNS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = rs1 & ((1 << 31) - 1) | !rs2 & (1 << 31);
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010000??????????010?????1010011")]
#[derive(Debug)]
struct FSGNJXS(InsnT);

impl FloatInsn for FSGNJXS {}

impl Execution for FSGNJXS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = rs1 & ((1 << 31) - 1) | (rs1 ^ rs2) & (1 << 31);
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b1010000??????????010?????1010011")]
#[derive(Debug)]
struct FEQS(InsnT);

impl FloatInsn for FEQS {}

impl FCompare<u32, F32Traits> for FEQS {}

impl Execution for FEQS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
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
#[code("0b1010000??????????001?????1010011")]
#[derive(Debug)]
struct FLTS(InsnT);

impl FloatInsn for FLTS {}

impl FCompare<u32, F32Traits> for FLTS {}

impl Execution for FLTS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
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
#[code("0b1010000??????????000?????1010011")]
#[derive(Debug)]
struct FLES(InsnT);

impl FloatInsn for FLES {}

impl FCompare<u32, F32Traits> for FLES {}

impl Execution for FLES {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
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
#[code("0b111000000000?????001?????1010011")]
#[derive(Debug)]
struct FCLASSS(InsnT);

impl FloatInsn for FCLASSS {}

impl FClass<u32, F32Traits> for FCLASSS {}

impl Execution for FCLASSS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        p.state().set_xreg(self.rd() as RegT, self.class(rs1));
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b111000000000?????000?????1010011")]
#[derive(Debug)]
struct FMVXW(InsnT);

impl FloatInsn for FMVXW {}

impl Execution for FMVXW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let data: RegT = f.freg(self.rs1() as RegT).bit_range(31, 0);
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b111100000000?????000?????1010011")]
#[derive(Debug)]
struct FMVWX(InsnT);

impl FloatInsn for FMVWX {}

impl Execution for FMVWX {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let data: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        f.set_freg(self.rd() as RegT, data as FRegT);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}