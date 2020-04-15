use crate::processor::insn_define::*;
use std::num::Wrapping;
use crate::processor::extensions::f::{ExtensionF, FRegT};
use crate::processor::extensions::Extension;
use std::rc::Rc;
use std::num::FpCategory;

trait F32Insn: InstructionImp {
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
    fn is_signaling_nan(f: f32) -> bool {
        let uf: u32 = f.to_bits();
        let signal_bit = 1 << 22;
        let signal_bit_clear = (uf & signal_bit) == 0;
        f.is_nan() && signal_bit_clear
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????010?????0000111")]
#[derive(Debug)]
struct FLW(InsnT);

impl F32Insn for FLW {}

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

trait FStore: F32Insn {
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

impl F32Insn for FSW {}

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

trait F32Compute: F32Insn {
    fn opt(&self, frs1: f64, frs2: f64) -> f64;
    fn nx(&self, fres: f64) -> bool {
        (fres - (fres as f32) as f64) != 0_f64 && !fres.is_nan()
    }
    fn dz(&self, _: f32, _: f32) -> bool {
        false
    }
    fn of(&self, fres: f64) -> bool {
        fres > std::f32::MAX as f64
    }
    fn uf(&self, fres: f64) -> bool {
        fres < std::f32::MIN as f64
    }
    fn nv(&self, frs1: f32, frs2: f32) -> bool {
        frs1.is_nan() || frs2.is_nan()
    }
    fn round(&self, rm: RegT, fres: f64) -> Result<f64, Exception> {
        let rounded = (fres as f32) as f64;
        match rm {
            0 => {
                Ok(fres)
            }
            1 => {
                Ok(if fres.abs() < rounded.abs() {
                    if fres.is_sign_positive() {
                        (fres - std::f64::EPSILON)
                    } else {
                        (fres + std::f64::EPSILON)
                    }
                } else {
                    fres
                })
            }
            2 => {
                Ok(if fres < rounded {
                    (fres - std::f64::EPSILON)
                } else {
                    fres
                })
            }
            3 => {
                Ok(if fres > rounded {
                    (fres + std::f64::EPSILON)
                } else {
                    fres
                })
            }
            4 => {
                Ok(if (fres - std::f32::MAX as f64).abs() > (fres - std::f32::MIN as f64).abs() {
                    std::f32::MIN as f64
                } else {
                    std::f32::MAX as f64
                })
            }
            _ => Err(Exception::IllegalInsn(self.ir()))
        }
    }
    fn compute(&self, f: &ExtensionF, rs1: u32, rs2: u32) -> Result<u32, Exception> {
        let frs1_32 = f32::from_bits(rs1);
        let frs2_32 = f32::from_bits(rs2);
        if self.nv(frs1_32, frs2_32) {
            f.csrs.fcsr_mut().set_nv(1)
        } else if self.dz(frs1_32, frs2_32) {
            f.csrs.fcsr_mut().set_dz(1)
        }
        let frs1: f64 = frs1_32 as f64;
        let frs2: f64 = frs2_32 as f64;
        let fres = self.opt(frs1, frs2);
        let need_round = self.nx(fres);
        if need_round {
            f.csrs.fcsr_mut().set_nx(1)
        }
        let rounded = if need_round {
            self.round(if self.rm() == 0x7 { f.csrs.fcsr().frm() } else { self.rm() }, fres)?
        } else {
            fres
        };
        if self.of(rounded) {
            f.csrs.fcsr_mut().set_of(1)
        } else if self.uf(rounded) {
            f.csrs.fcsr_mut().set_uf(1)
        }
        Ok(if rounded.is_nan() {
            std::f32::NAN.to_bits()
        } else {
            (rounded as f32).to_bits()
        })
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000000??????????????????1010011")]
#[derive(Debug)]
struct FADDS(InsnT);

impl F32Insn for FADDS {}

impl F32Compute for FADDS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1 + frs2
    }
    fn nv(&self, frs1: f32, frs2: f32) -> bool {
        frs1.is_nan() || frs2.is_nan() ||
            frs1 == std::f32::INFINITY && frs2 == std::f32::NEG_INFINITY ||
            frs2 == std::f32::INFINITY && frs1 == std::f32::NEG_INFINITY
    }
}

impl Execution for FADDS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2)?;
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

impl F32Insn for FSUBS {}

impl F32Compute for FSUBS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1 - frs2
    }
    fn nv(&self, frs1: f32, frs2: f32) -> bool {
        frs1.is_nan() || frs2.is_nan() ||
            frs1 == std::f32::INFINITY && frs2 == std::f32::INFINITY ||
            frs2 == std::f32::NEG_INFINITY && frs1 == std::f32::NEG_INFINITY
    }
}

impl Execution for FSUBS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2)?;
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

impl F32Insn for FMULS {}

impl F32Compute for FMULS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1 * frs2
    }
    fn nv(&self, frs1: f32, frs2: f32) -> bool {
        frs1.is_nan() || frs2.is_nan() ||
            frs1.is_infinite() && frs2 == 0_f32 ||
            frs1 == 0_f32 && frs2.is_infinite()
    }
}

impl Execution for FMULS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2)?;
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

impl F32Insn for FDIVS {}

impl F32Compute for FDIVS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        frs1 / frs2
    }
    fn nv(&self, frs1: f32, frs2: f32) -> bool {
        frs1.is_nan() || frs2.is_nan() ||
            frs1.is_infinite() && frs2.is_infinite() ||
            frs1 == 0_f32 && frs2 == 0_f32
    }
    fn dz(&self, _: f32, frs2: f32) -> bool {
        frs2 == 0_f32
    }
}

impl Execution for FDIVS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2)?;
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

impl F32Insn for FSQRTS {}

impl F32Compute for FSQRTS {
    fn opt(&self, frs1: f64, _: f64) -> f64 {
        frs1.sqrt()
    }
    fn nv(&self, frs1: f32, _: f32) -> bool {
        frs1.is_nan() || frs1 < 0_f32
    }
}

impl Execution for FSQRTS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, 0)?;
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

impl F32Insn for FMINS {}

impl F32Compute for FMINS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        if frs1 == frs2 && frs1.is_sign_negative() {
            frs1.min(frs2)
        } else {
            frs2.min(frs1)
        }
    }
    fn nv(&self, frs1: f32, frs2: f32) -> bool {
        Self::is_signaling_nan(frs1) || Self::is_signaling_nan(frs2)
    }
}

impl Execution for FMINS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2)?;
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

impl F32Insn for FMAXS {}

impl F32Compute for FMAXS {
    fn opt(&self, frs1: f64, frs2: f64) -> f64 {
        if frs1 == frs2 && frs1.is_sign_positive() {
            frs1.max(frs2)
        } else {
            frs2.max(frs1)
        }
    }
    fn nv(&self, frs1: f32, frs2: f32) -> bool {
        Self::is_signaling_nan(frs1) || Self::is_signaling_nan(frs2)
    }
}

impl Execution for FMAXS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let res = self.compute(f.deref(), rs1, rs2)?;
        f.set_freg(self.rd() as RegT, res as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait F32FMA: F32Insn {
    fn opt(&self, frs1: f64, frs2: f64, frs3: f64) -> f64;
    fn rs3(&self) -> InsnT {
        self.ir().bit_range(31, 27)
    }
    fn nx(&self, fres: f64) -> bool {
        (fres - (fres as f32) as f64) != 0_f64 && !fres.is_nan()
    }
    fn of(&self, fres: f64) -> bool {
        fres > std::f32::MAX as f64
    }
    fn uf(&self, fres: f64) -> bool {
        fres < std::f32::MIN as f64
    }
    fn nv(&self, frs1: f64, frs2: f64, frs3: f64) -> bool {
        frs3.is_nan() && (frs1.is_infinite() && frs2 == 0_f64 || frs1 == 0_f64 && frs2.is_infinite())
    }
    fn round(&self, rm: RegT, fres: f64) -> Result<f64, Exception> {
        let rounded = (fres as f32) as f64;
        match rm {
            0 => {
                Ok(fres)
            }
            1 => {
                Ok(if fres.abs() < rounded.abs() {
                    if fres.is_sign_positive() {
                        (fres - std::f64::EPSILON)
                    } else {
                        (fres + std::f64::EPSILON)
                    }
                } else {
                    fres
                })
            }
            2 => {
                Ok(if fres < rounded {
                    (fres - std::f64::EPSILON)
                } else {
                    fres
                })
            }
            3 => {
                Ok(if fres > rounded {
                    (fres + std::f64::EPSILON)
                } else {
                    fres
                })
            }
            4 => {
                Ok(if (fres - std::f32::MAX as f64).abs() > (fres - std::f32::MIN as f64).abs() {
                    std::f32::MIN as f64
                } else {
                    std::f32::MAX as f64
                })
            }
            _ => Err(Exception::IllegalInsn(self.ir()))
        }
    }
    fn compute(&self, f: &ExtensionF, rs1: u32, rs2: u32, rs3: u32) -> Result<u32, Exception> {
        let frs1: f64 = f32::from_bits(rs1) as f64;
        let frs2: f64 = f32::from_bits(rs2) as f64;
        let frs3: f64 = f32::from_bits(rs3) as f64;
        let fres = self.opt(frs1, frs2, frs3);
        let need_round = self.nx(fres);
        if self.nv(frs1, frs2, frs3) {
            f.csrs.fcsr_mut().set_nv(1)
        } else if need_round {
            f.csrs.fcsr_mut().set_nx(1)
        }
        let rounded = if need_round {
            self.round(if self.rm() == 0x7 { f.csrs.fcsr().frm() } else { self.rm() }, fres)?
        } else {
            fres
        };
        if self.of(rounded) {
            f.csrs.fcsr_mut().set_of(1)
        } else if self.uf(rounded) {
            f.csrs.fcsr_mut().set_uf(1)
        }
        Ok(if rounded.is_nan() {
            std::f32::NAN.to_bits()
        } else {
            (rounded as f32).to_bits()
        })
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b?????00??????????????????1000011")]
#[derive(Debug)]
struct FMADDS(InsnT);

impl F32Insn for FMADDS {}

impl F32FMA for FMADDS {
    fn opt(&self, frs1: f64, frs2: f64, frs3: f64) -> f64 {
        frs1.mul_add(frs2, frs3)
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

impl F32Insn for FMSUBS {}

impl F32FMA for FMSUBS {
    fn opt(&self, frs1: f64, frs2: f64, frs3: f64) -> f64 {
        frs1.mul_add(frs2, -frs3)
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

impl F32Insn for FMNSUBS {}

impl F32FMA for FMNSUBS {
    fn opt(&self, frs1: f64, frs2: f64, frs3: f64) -> f64 {
        -frs1.mul_add(frs2, -frs3)
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

impl F32Insn for FMNADDS {}

impl F32FMA for FMNADDS {
    fn opt(&self, frs1: f64, frs2: f64, frs3: f64) -> f64 {
        -frs1.mul_add(frs2, frs3)
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


trait F32ToInt: F32Insn {
    fn round(&self, rm: RegT, fres: f32) -> Result<f32, Exception> {
        match rm {
            0 => {
                Ok(fres.round())
            }
            1 => {
                Ok(fres.trunc())
            }
            2 => {
                Ok(fres.floor())
            }
            3 => {
                Ok(fres.ceil())
            }
            4 => {
                Ok(if (fres - std::f32::MAX).abs() > (fres - std::f32::MIN).abs() {
                    std::f32::MIN
                } else {
                    std::f32::MAX
                })
            }
            _ => Err(Exception::IllegalInsn(self.ir()))
        }
    }
    fn convert(&self, f: &ExtensionF, rs1: u32) -> Result<f32, Exception> {
        let fres: f32 = f32::from_bits(rs1);
        let need_round = fres.fract() != 0_f32;
        let rounded = if need_round {
            f.csrs.fcsr_mut().set_nx(1);
            self.round(if self.rm() == 0x7 { f.csrs.fcsr().frm() } else { self.rm() }, fres)
        } else {
            Ok(fres)
        };
        rounded
    }
}


#[derive(Instruction)]
#[format(R)]
#[code("0b110000000000?????????????1010011")]
#[derive(Debug)]
struct FCVTWS(InsnT);

impl F32Insn for FCVTWS {}

impl F32ToInt for FCVTWS {}

impl Execution for FCVTWS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1)?;
        let res = if fres.is_nan() || fres > std::i32::MAX as f32 || fres.is_infinite() && fres.is_sign_positive() {
            f.csrs.fcsr_mut().set_nv(1);
            (1u32 << 31) - 1
        } else if fres < std::i32::MIN as f32 || fres.is_infinite() && fres.is_sign_negative() {
            f.csrs.fcsr_mut().set_nv(1);
            1u32 << 31
        } else {
            fres as i32 as u32
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

impl F32Insn for FCVTWUS {}

impl F32ToInt for FCVTWUS {}

impl Execution for FCVTWUS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1)?;
        let res = if fres.is_nan() || fres > std::u32::MAX as f32 || fres.is_infinite() && fres.is_sign_positive() {
            f.csrs.fcsr_mut().set_nv(1);
            -1i32 as u32
        } else if fres < std::u32::MIN as f32 || fres.is_infinite() && fres.is_sign_negative() {
            f.csrs.fcsr_mut().set_nv(1);
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

impl F32Insn for FCVTLS {}

impl F32ToInt for FCVTLS {}

impl Execution for FCVTLS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1)?;
        let res = if fres.is_nan() || fres > std::i64::MAX as f32 || fres.is_infinite() && fres.is_sign_positive() {
            f.csrs.fcsr_mut().set_nv(1);
            (1u64 << 63) - 1
        } else if fres < std::i64::MIN as f32 || fres.is_infinite() && fres.is_sign_negative() {
            f.csrs.fcsr_mut().set_nv(1);
            1u64 << 63
        } else {
            fres as i64 as u64
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

impl F32Insn for FCVTLUS {}

impl F32ToInt for FCVTLUS {}

impl Execution for FCVTLUS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1)?;
        let res = if fres.is_nan() || fres > std::u64::MAX as f32 || fres.is_infinite() && fres.is_sign_positive() {
            f.csrs.fcsr_mut().set_nv(1);
            -1i64 as u64
        } else if fres < std::u64::MIN as f32 || fres.is_infinite() && fres.is_sign_negative() {
            f.csrs.fcsr_mut().set_nv(1);
            0
        } else {
            fres as u64
        };
        p.state().set_xreg(self.rd() as RegT, res as RegT & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait IntToF32: F32Insn {
    fn round(&self, rm: RegT, fres: f64) -> Result<f64, Exception> {
        let rounded = (fres as f32) as f64;
        match rm {
            0 => {
                Ok(fres)
            }
            1 => {
                Ok(if fres.abs() < rounded.abs() {
                    if fres.is_sign_positive() {
                        (fres - std::f64::EPSILON)
                    } else {
                        (fres + std::f64::EPSILON)
                    }
                } else {
                    fres
                })
            }
            2 => {
                Ok(if fres < rounded {
                    (fres - std::f64::EPSILON)
                } else {
                    fres
                })
            }
            3 => {
                Ok(if fres > rounded {
                    (fres + std::f64::EPSILON)
                } else {
                    fres
                })
            }
            4 => {
                Ok(if (fres - std::f32::MAX as f64).abs() > (fres - std::f32::MIN as f64).abs() {
                    std::f32::MIN as f64
                } else {
                    std::f32::MAX as f64
                })
            }
            _ => Err(Exception::IllegalInsn(self.ir()))
        }
    }
    fn convert(&self, f: &ExtensionF, rs1: RegT, signed: bool) -> Result<f32, Exception> {
        let fres = if signed { rs1 as SRegT as f64 } else { rs1 as f64 };
        let rounded = self.round(if self.rm() == 0x7 { f.csrs.fcsr().frm() } else { self.rm() }, fres)?;
        if rounded > std::f32::MAX as f64 {
            f.csrs.fcsr_mut().set_of(1)
        } else if rounded < std::f32::MIN as f64 {
            f.csrs.fcsr_mut().set_uf(1)
        }
        Ok(if rounded.is_nan() {
            std::f32::NAN
        } else {
            rounded as f32
        })
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000000?????????????1010011")]
#[derive(Debug)]
struct FCVTSW(InsnT);

impl F32Insn for FCVTSW {}

impl IntToF32 for FCVTSW {}

impl Execution for FCVTSW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: RegT = sext(p.state().xreg(self.rs1() as RegT), 32);
        let fres = self.convert(f.deref(), rs1, true)?;
        f.set_freg(self.rd() as RegT, fres.to_bits() as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000001?????????????1010011")]
#[derive(Debug)]
struct FCVTSWU(InsnT);

impl F32Insn for FCVTSWU {}

impl IntToF32 for FCVTSWU {}

impl Execution for FCVTSWU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        let fres = self.convert(f.deref(), rs1, false)?;
        f.set_freg(self.rd() as RegT, fres.to_bits() as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000010?????????????1010011")]
#[derive(Debug)]
struct FCVTSL(InsnT);

impl F32Insn for FCVTSL {}

impl IntToF32 for FCVTSL {}

impl Execution for FCVTSL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT);
        let fres = self.convert(f.deref(), rs1, true)?;
        f.set_freg(self.rd() as RegT, fres.to_bits() as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b110100000011?????????????1010011")]
#[derive(Debug)]
struct FCVTSLU(InsnT);

impl F32Insn for FCVTSLU {}

impl IntToF32 for FCVTSLU {}

impl Execution for FCVTSLU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let f = self.get_f_ext(p)?;
        let rs1: RegT = p.state().xreg(self.rs1() as RegT);
        let fres = self.convert(f.deref(), rs1, false)?;
        f.set_freg(self.rd() as RegT, fres.to_bits() as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0010000??????????000?????1010011")]
#[derive(Debug)]
struct FSGNJS(InsnT);

impl F32Insn for FSGNJS {}

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

impl F32Insn for FSGNJNS {}

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

impl F32Insn for FSGNJXS {}

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

impl F32Insn for FEQS {}

impl Execution for FEQS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let frs1 = f32::from_bits(rs1);
        let frs2 = f32::from_bits(rs2);
        if Self::is_signaling_nan(frs1) || Self::is_signaling_nan(frs2) {
            f.csrs.fcsr_mut().set_nv(1)
        }
        if frs1 == frs2 {
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

impl F32Insn for FLTS {}

impl Execution for FLTS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let frs1 = f32::from_bits(rs1);
        let frs2 = f32::from_bits(rs2);
        if frs1.is_nan() || frs1.is_nan() {
            f.csrs.fcsr_mut().set_nv(1)
        }
        if frs1 < frs2 {
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

impl F32Insn for FLES {}

impl Execution for FLES {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let rs2: u32 = f.freg(self.rs2() as RegT).bit_range(31, 0);
        let frs1 = f32::from_bits(rs1);
        let frs2 = f32::from_bits(rs2);
        if frs1.is_nan() || frs1.is_nan() {
            f.csrs.fcsr_mut().set_nv(1)
        }
        if frs1 <= frs2 {
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

impl F32Insn for FCLASSS {}

impl Execution for FCLASSS {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let rs1: u32 = f.freg(self.rs1() as RegT).bit_range(31, 0);
        let frs1 = f32::from_bits(rs1);
        match frs1.classify() {
            FpCategory::Infinite => {
                if frs1.is_sign_negative() {
                    p.state().set_xreg(self.rd() as RegT, 1)
                } else {
                    p.state().set_xreg(self.rd() as RegT, 1 << 7)
                }
            }
            FpCategory::Normal => {
                if frs1.is_sign_negative() {
                    p.state().set_xreg(self.rd() as RegT, 1 << 1)
                } else {
                    p.state().set_xreg(self.rd() as RegT, 1 << 6)
                }
            }
            FpCategory::Subnormal => {
                if frs1.is_sign_negative() {
                    p.state().set_xreg(self.rd() as RegT, 1 << 2)
                } else {
                    p.state().set_xreg(self.rd() as RegT, 1 << 5)
                }
            }
            FpCategory::Zero => {
                if frs1.is_sign_negative() {
                    p.state().set_xreg(self.rd() as RegT, 1 << 3)
                } else {
                    p.state().set_xreg(self.rd() as RegT, 1 << 4)
                }
            }
            FpCategory::Nan => {
                if Self::is_signaling_nan(frs1) {
                    p.state().set_xreg(self.rd() as RegT, 1 << 8)
                } else {
                    p.state().set_xreg(self.rd() as RegT, 1 << 9)
                }
            }
        }
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b111000000000?????000?????1010011")]
#[derive(Debug)]
struct FMVXW(InsnT);

impl F32Insn for FMVXW {}

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

impl F32Insn for FMVWX {}

impl Execution for FMVWX {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let data: RegT = p.state().xreg(self.rs1() as RegT).bit_range(31, 0);
        f.set_freg(self.rd() as RegT, data as FRegT);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}