use crate::processor::insn_define::*;
use std::num::Wrapping;
use crate::processor::extensions::f::{ExtensionF, FRegT};
use crate::processor::extensions::Extension;
use std::rc::Rc;
use std::num::FpCategory;
use std::ops::AddAssign;

trait F64Insn: InstructionImp {
    fn get_f_ext(&self, p: &Processor) -> Result<Rc<ExtensionF>, Exception> {
        p.state().check_extension('f')?;
        p.state().check_extension('d')?;
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
    fn is_signaling_nan(f: f64) -> bool {
        let uf: u64 = f.to_bits();
        let signal_bit = 1 << 51;
        let signal_bit_clear = (uf & signal_bit) == 0;
        f.is_nan() && signal_bit_clear
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b?????????????????011?????0000111")]
#[derive(Debug)]
struct FLD(InsnT);

impl F64Insn for FLD {}

impl Execution for FLD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let offset: Wrapping<RegT> = Wrapping(sext(self.imm() as RegT, self.imm_len()));
        let data = p.load_store().load_double_word((base + offset).0, p.mmu())?;
        f.set_freg(self.rd() as RegT, data as FRegT & f.flen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

trait FStore: F64Insn {
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
#[code("0b?????????????????011?????0100111")]
#[derive(Debug)]
struct FSD(InsnT);

impl F64Insn for FSD {}

impl FStore for FSD {}

impl Execution for FSD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let f = self.get_f_ext(p)?;
        let base: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let data = f.freg(self.src());
        p.load_store().store_double_word((base + self.offset()).0, data as RegT, p.mmu())?;
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}
