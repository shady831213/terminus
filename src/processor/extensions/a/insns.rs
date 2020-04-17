use crate::processor::insn_define::*;
use crate::processor::extensions::a::ExtensionA;
use std::rc::Rc;
use crate::processor::extensions::Extension;

pub trait AMOInsn: InstructionImp {
    fn get_a_ext(&self, p: &Processor) -> Result<Rc<ExtensionA>, Exception> {
        p.state().check_extension('a')?;
        if let Some(Extension::A(a)) = p.state().extensions().get(&'a') {
            Ok(a.clone())
        } else {
            Err(Exception::IllegalInsn(self.ir()))
        }
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00010??00000?????010?????0101111")]
#[derive(Debug)]
struct LRW(InsnT);

impl AMOInsn for LRW {}

impl Execution for LRW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let a = self.get_a_ext(p)?;
        a.clr_lc_res();
        p.load_store().release();
        let addr = p.state().xreg(self.rs1() as RegT);
        let success = p.load_store().acquire(addr, 4, p.mmu())?;
        let data = p.load_store().load_word(addr, p.mmu())?;
        if success {
            a.set_lc_res(addr, 4, p.state().insns_cnt())
        }
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00010??00000?????011?????0101111")]
#[derive(Debug)]
struct LRD(InsnT);

impl AMOInsn for LRD {}

impl Execution for LRD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let a = self.get_a_ext(p)?;
        a.clr_lc_res();
        p.load_store().release();
        let addr = p.state().xreg(self.rs1() as RegT);
        let success = p.load_store().acquire(addr, 8, p.mmu())?;
        let data = p.load_store().load_double_word(addr, p.mmu())?;
        if success {
            a.set_lc_res(addr, 8, p.state().insns_cnt())
        }
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00011????????????010?????0101111")]
#[derive(Debug)]
struct SCW(InsnT);

impl AMOInsn for SCW {}

impl Execution for SCW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        let a = self.get_a_ext(p)?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let data = p.state().xreg(self.rs2() as RegT);
        let success = if let Some(lc_res) = a.lc_res().deref() {
            if addr != lc_res.addr || lc_res.len != 4 {
                false
            } else {
                p.load_store().check_lock(addr, 4, p.mmu())?
            }
        } else {
            false
        };
        if success {
            p.load_store().store_word(addr, data, p.mmu())?
        }
        a.clr_lc_res();
        p.load_store().release();
        p.state().set_xreg(self.rd() as RegT, (!success) as RegT);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00011????????????011?????0101111")]
#[derive(Debug)]
struct SCD(InsnT);

impl AMOInsn for SCD {}

impl Execution for SCD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let a = self.get_a_ext(p)?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let data = p.state().xreg(self.rs2() as RegT);
        let success = if let Some(lc_res) = a.lc_res().deref() {
            if addr != lc_res.addr || lc_res.len != 8 {
                false
            } else {
                p.load_store().check_lock(addr, 8, p.mmu())?
            }
        } else {
            false
        };
        if success {
            p.load_store().store_double_word(addr, data, p.mmu())?
        }
        a.clr_lc_res();
        p.load_store().release();
        p.state().set_xreg(self.rd() as RegT, (!success) as RegT);
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}