use terminus_global::*;
use terminus_macros::*;
use terminus_proc_macros::Instruction;
use crate::processor::Processor;
use crate::processor::trap::Exception;
use crate::processor::insn::*;
use crate::processor::decode::*;
use crate::linkme::*;
use std::num::Wrapping;
use crate::processor::extensions::f::{ExtensionF, FRegT};
use crate::processor::extensions::Extension;
use std::rc::Rc;

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

trait F32Compute: FloatInsn {
    fn opt(&self, frs1: f64, frs2: f64) -> f64;
    fn nx(&self, fres: f64) -> bool {
        (fres - (fres as f32) as f64) != 0_f64
    }
    fn dz(&self, _: f64, _: f64) -> bool {
        false
    }
    fn of(&self, fres: f64) -> bool {
        fres > std::f32::MAX as f64
    }
    fn uf(&self, fres: f64) -> bool {
        fres < std::f32::MIN as f64
    }
    fn nv(&self, frs1: f64, frs2: f64) -> bool {
        frs1.is_nan() || frs2.is_nan()
    }
    fn round(&self, rm: RegT, fres: f64) -> f64 {
        let rounded = (fres as f32) as f64;
        match rm {
            0 => {
                fres
            }
            1 => {
                if fres.abs() < rounded.abs() {
                    if fres.is_sign_positive() {
                        (fres - std::f64::EPSILON)
                    } else {
                        (fres + std::f64::EPSILON)
                    }
                } else {
                    fres
                }
            }
            2 => {
                if fres < rounded {
                    (fres - std::f64::EPSILON)
                } else {
                    fres
                }
            }
            3 => {
                if fres > rounded {
                    (fres + std::f64::EPSILON)
                } else {
                    fres
                }
            }
            _ => unreachable!()
        }
    }
    fn compute(&self, f: &ExtensionF, rs1: u32, rs2: u32) -> u32 {
        let frs1: f64 = f32::from_bits(rs1) as f64;
        let frs2: f64 = f32::from_bits(rs2) as f64;
        let fres = self.opt(frs1, frs2);
        let need_round = self.nx(fres);
        if self.nv(frs1, frs2) {
            f.csrs.fcsr_mut().set_nv(1)
        } else if self.dz(frs1, frs2) {
            f.csrs.fcsr_mut().set_dz(1)
        } else if need_round {
            f.csrs.fcsr_mut().set_nx(1)
        }
        let rounded = if need_round {
            self.round(if f.csrs.fcsr().frm() == 0x7 { self.rm() } else { f.csrs.fcsr().frm() }, fres)
        } else {
            fres
        };
        if self.of(rounded) {
            f.csrs.fcsr_mut().set_of(1)
        } else if self.uf(rounded) {
            f.csrs.fcsr_mut().set_uf(1)
        }
        if rounded.is_nan() {
            std::f32::NAN.to_bits()
        } else {
            (rounded as f32).to_bits()
        }
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????????????1010011")]
#[derive(Debug)]
struct FADDS(InsnT);

impl FloatInsn for FADDS {}

impl F32Compute for FADDS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1 + frs2
    }
    fn nv(&self, frs1: f64, frs2: f64) -> bool {
        frs1.is_nan() || frs2.is_nan() ||
            frs1 == std::f64::INFINITY && frs2 == std::f64::NEG_INFINITY ||
            frs2 == std::f64::INFINITY && frs1 == std::f64::NEG_INFINITY
    }
}

impl Execution for FADDS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2);
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

impl F32Compute for FSUBS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1 - frs2
    }
    fn nv(&self, frs1: f64, frs2: f64) -> bool {
        frs1.is_nan() || frs2.is_nan() ||
            frs1 == std::f64::INFINITY && frs2 == std::f64::INFINITY ||
            frs2 == std::f64::NEG_INFINITY && frs1 == std::f64::NEG_INFINITY
    }
}

impl Execution for FSUBS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2);
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

impl F32Compute for FMULS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1 * frs2
    }
    fn nv(&self, frs1: f64, frs2: f64) -> bool {
        frs1.is_nan() || frs2.is_nan() ||
            frs1.is_infinite() && frs2 == 0_f64 ||
            frs1 == 0_f64 && frs2.is_infinite()
    }
}

impl Execution for FMULS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2);
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

impl F32Compute for FDIVS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1 / frs2
    }
    fn nv(&self, frs1: f64, frs2: f64) -> bool {
        frs1.is_nan() || frs2.is_nan() ||
            frs1.is_infinite() && frs2.is_infinite() ||
            frs1 == 0_f64 && frs2 == 0_f64
    }
    fn dz(&self, _: f64, frs2: f64) -> bool {
        frs2 == 0_f64
    }
}

impl Execution for FDIVS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2);
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

impl F32Compute for FSQRTS {
    fn opt(&self, frs1: f64, _: f64) -> f64 {
        frs1.sqrt()
    }
    fn nv(&self, frs1: f64, _: f64) -> bool {
        frs1.is_nan() || frs1 < 0_f64
    }
}

impl Execution for FSQRTS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, 0);
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

impl F32Compute for FMINS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1.min(frs2)
    }
}

impl Execution for FMINS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2);
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

impl F32Compute for FMAXS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1.max(frs2)
    }
}

impl Execution for FMAXS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2);
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait F32ToInt: FloatInsn {
    fn round(&self, rm: RegT, fres: f64) -> f64 {
        match rm {
            0 => {
                fres.round()
            }
            1 => {
                fres.trunc()
            }
            2 => {
                fres.floor()
            }
            3 => {
                fres.ceil()
            }
            _ => unreachable!()
        }
    }
    fn convert(&self, f: &ExtensionF, rs1: u32) -> f64 {
        let fres: f64 = f32::from_bits(rs1) as f64;
        if fres.is_nan() || fres.is_infinite() {
            f.csrs.fcsr_mut().set_nv(1)
        }
        let need_round = fres.fract() != 0_f64;
        let rounded = if need_round {
            f.csrs.fcsr_mut().set_nx(1);
            self.round(if f.csrs.fcsr().frm() == 0x7 { self.rm() } else { f.csrs.fcsr().frm() }, fres)
        } else {
            fres
        };
        if rounded > std::f32::MAX as f64 {
            f.csrs.fcsr_mut().set_of(1)
        } else if rounded < std::f32::MIN as f64 {
            f.csrs.fcsr_mut().set_uf(1)
        }
        rounded
    }
}


#[derive(Instruction)]
#[format(R)]
#[code("0b110000000000?????????????1010011")]
#[derive(Debug)]
struct FCVTWS(InsnT);

impl FloatInsn for FCVTWS {}

impl F32ToInt for FCVTWS {}

impl Execution for FCVTWS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1);
        let res = if fres.is_nan() || fres >= std::f32::MAX as f64 {
            (1u32 << 31) - 1
        } else if fres <= std::f32::MIN as f64 {
            1u32 << 31
        } else {
            fres as u32
        };
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

impl F32ToInt for FCVTWUS {}

impl Execution for FCVTWUS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1);
        let res = if fres.is_nan() || fres >= std::f32::MAX as f64 {
            -1i32 as u32
        } else if fres <= std::f32::MIN as f64 {
            0
        } else {
            fres as u32
        };
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

impl F32ToInt for FCVTLS {}

impl Execution for FCVTLS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1);
        let res = if fres.is_nan() || fres >= std::f32::MAX as f64 {
            (1u64 << 63) - 1
        } else if fres <= std::f32::MIN as f64 {
            1u64 << 63
        } else {
            fres as u64
        };
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

impl F32ToInt for FCVTLUS {}

impl Execution for FCVTLUS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1);
        let res = if fres.is_nan() || fres >= std::f32::MAX as f64 {
            -1i64 as u64
        } else if fres <= std::f32::MIN as f64 {
            0
        } else {
            fres as u64
        };
        p.state().set_xreg(self.rd() as RegT, res as RegT & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait IntToF32: FloatInsn {
    fn round(&self, rm: RegT, fres: f64) -> f64 {
        let rounded = (fres as f32) as f64;
        match rm {
            0 => {
                fres
            }
            1 => {
                if fres.abs() < rounded.abs() {
                    if fres.is_sign_positive() {
                        (fres - std::f64::EPSILON)
                    } else {
                        (fres + std::f64::EPSILON)
                    }
                } else {
                    fres
                }
            }
            2 => {
                if fres < rounded {
                    (fres - std::f64::EPSILON)
                } else {
                    fres
                }
            }
            3 => {
                if fres > rounded {
                    (fres + std::f64::EPSILON)
                } else {
                    fres
                }
            }
            _ => unreachable!()
        }
    }
    fn convert(&self, f: &ExtensionF, rs1: RegT, signed:bool) -> f32 {
        let fres = if signed {rs1 as SRegT as f64} else {rs1 as f64};
        let rounded = self.round(if f.csrs.fcsr().frm() == 0x7 { self.rm() } else { f.csrs.fcsr().frm() }, fres);
        if rounded > std::f32::MAX as f64 {
            f.csrs.fcsr_mut().set_of(1)
        } else if rounded < std::f32::MIN as f64 {
            f.csrs.fcsr_mut().set_uf(1)
        }
        if rounded.is_nan() {
            std::f32::NAN
        } else {
            rounded as f32
        }
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000000?????????????1010011")]
#[derive(Debug)]
struct FCVTSW(InsnT);

impl FloatInsn for FCVTSW {}

impl IntToF32 for FCVTSW {}

impl Execution for FCVTSW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: RegT = sext(p.state().xreg(self.rs1() as RegT), 32);
        let fres = self.convert(f.deref(), rs1, true);
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

impl IntToF32 for FCVTSWU {}

impl Execution for FCVTSWU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1, false);
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

impl IntToF32 for FCVTSL {}

impl Execution for FCVTSL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT);
        let fres = self.convert(f.deref(), rs1, true);
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

impl IntToF32 for FCVTSLU {}

impl Execution for FCVTSLU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT);
        let fres = self.convert(f.deref(), rs1, false);
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
        let frs1 = f32::from_bits(rs1);
        let frs2 = f32::from_bits(rs2);
        let fres = frs1.copysign(frs2);
        f.set_freg(self.rd() as RegT, fres as FRegT & f.flen.mask());
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
        let frs1 = f32::from_bits(rs1);
        let frs2 = f32::from_bits(rs2);
        let fres = frs1.copysign(-frs2);
        f.set_freg(self.rd() as RegT, fres as FRegT & f.flen.mask());
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
        let frs1 = f32::from_bits(rs1);
        let frs2 = f32::from_bits(rs2);
        let sign1 = frs1.signum();
        let sign2 = frs2.signum();
        let sign = if sign1 == sign2 {
            -1_f32
        } else {
            1_f32
        };
        let fres = frs1.copysign(sign);
        f.set_freg(self.rd() as RegT, fres as FRegT & f.flen.mask());
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