extern crate simple_soft_float;

use crate::prelude::*;
use crate::processor::extensions::f::ExtensionF;
use crate::processor::extensions::Extension;
use std::rc::Rc;
use simple_soft_float::{RoundingMode, StatusFlags, FloatClass, FloatTraits, Float, FloatBitsType};
use std::cmp::Ordering;
use std::num::Wrapping;

pub use simple_soft_float::{F64, F32, Sign, F64Traits, F32Traits, FPState};

pub trait FloatInsn: InstructionImp {
    fn get_f_ext(&self, p: &Processor) -> Result<Rc<ExtensionF>, Exception> {
        p.state().check_extension('f')?;
        if let Extension::F(ref f) = p.state().get_extension('f') {
            if f.dirty() == 0 {
                Err(Exception::IllegalInsn(p.state().ir()))
            } else {
                Ok(f.clone())
            }
        } else {
            Err(Exception::IllegalInsn(p.state().ir()))
        }
    }
    fn rm(&self, code: InsnT) -> RegT {
        ((code >> 12) & 0x7) as RegT
    }

    fn rs3(&self, code: InsnT) -> InsnT {
        (code >> 27) & 0x1f
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


pub trait FStore: FloatInsn {
    fn offset(&self, code: InsnT) -> Wrapping<RegT> {
        let high: RegT = ((self.imm(code) >> 5) & 0x7f) as RegT;
        let low = self.rd(code) as RegT;
        Wrapping(sext(high << 5 | low, self.imm_len()))
    }
    fn src(&self, code: InsnT) -> InsnT {
        self.imm(code) & 0x1f
    }
}

pub trait FCompute<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    fn opt(&self, ir: InsnT, frs1: Float<FpTrait>, frs2: Float<FpTrait>, frs3: Float<FpTrait>, state: &mut FPState) -> Float<FpTrait>;
    fn compute(&self, ir: InsnT, f: &ExtensionF, rs1: Bits, rs2: Bits, rs3: Bits) -> Result<Bits, Exception> {
        let mut fp_state = FPState::default();
        fp_state.rounding_mode = if let Some(rm) = Self::rm_from_bits(f.csrs.frm().get()) {
            rm
        } else {
            if self.rm(ir) == 7 { return Err(Exception::IllegalInsn(ir)); } else { RoundingMode::default() }
        };
        let frs1 = Float::<FpTrait>::from_bits(rs1);
        let frs2 = Float::<FpTrait>::from_bits(rs2);
        let frs3 = Float::<FpTrait>::from_bits(rs3);
        let fres = self.opt(ir, frs1, frs2, frs3, &mut fp_state);
        f.csrs.fflags_mut().set(Self::status_flags_to_bits(&fp_state.status_flags));
        Ok(*fres.bits())
    }
}


pub trait FToX<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    type T;
    fn opt(&self, ir: InsnT, frs1: Float<FpTrait>, state: &mut FPState) -> Self::T;
    fn convert(&self, ir: InsnT, f: &ExtensionF, rs1: Bits) -> Result<Self::T, Exception> {
        let mut fp_state = FPState::default();
        fp_state.rounding_mode = if let Some(rm) = Self::rm_from_bits(f.csrs.frm().get()) {
            rm
        } else {
            if self.rm(ir) == 7 { return Err(Exception::IllegalInsn(ir)); } else { RoundingMode::default() }
        };
        let fres = Float::<FpTrait>::from_bits(rs1);
        let res: Self::T = self.opt(ir, fres, &mut fp_state);
        f.csrs.fflags_mut().set(Self::status_flags_to_bits(&fp_state.status_flags));
        Ok(res)
    }
}

pub trait XToF<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    type T;
    fn opt(&self, ir: InsnT, rs1: Self::T, state: &mut FPState) -> Float<FpTrait>;
    fn convert(&self, ir: InsnT, f: &ExtensionF, rs1: Self::T) -> Result<Bits, Exception> {
        let mut fp_state = FPState::default();
        fp_state.rounding_mode = if let Some(rm) = Self::rm_from_bits(f.csrs.frm().get()) {
            rm
        } else {
            if self.rm(ir) == 7 { return Err(Exception::IllegalInsn(ir)); } else { RoundingMode::default() }
        };
        let res = self.opt(ir, rs1, &mut fp_state);
        f.csrs.fflags_mut().set(Self::status_flags_to_bits(&fp_state.status_flags));
        Ok(*res.bits())
    }
}

pub trait FCompare<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
    fn compare(&self, ir: InsnT, f: &ExtensionF, rs1: Bits, rs2: Bits, check_nan: bool) -> Result<Option<Ordering>, Exception> {
        let mut fp_state = FPState::default();
        fp_state.rounding_mode = if let Some(rm) = Self::rm_from_bits(f.csrs.frm().get()) {
            rm
        } else {
            if self.rm(ir) == 7 { return Err(Exception::IllegalInsn(ir)); } else { RoundingMode::default() }
        };
        let frs1 = Float::<FpTrait>::from_bits(rs1);
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

pub trait FClass<Bits: FloatBitsType + Copy, FpTrait: FloatTraits<Bits=Bits> + Default>: FloatInsn {
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

