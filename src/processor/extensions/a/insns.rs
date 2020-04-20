use crate::prelude::*;
use crate::processor::extensions::a::ExtensionA;
use std::rc::Rc;
use crate::processor::extensions::Extension;
use std::num::Wrapping;
use std::cmp::{min, max};

pub trait LRSCInsn: InstructionImp {
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

impl LRSCInsn for LRW {}

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

impl LRSCInsn for LRD {}

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

impl LRSCInsn for SCW {}

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

impl LRSCInsn for SCD {}

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

#[derive(Instruction)]
#[format(R)]
#[code("0b00001????????????010?????0101111")]
#[derive(Debug)]
struct AMOSWAPW(InsnT);

impl Execution for AMOSWAPW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT);
        let data = p.load_store().amo_word(addr, |_| { src as u32 }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00001????????????011?????0101111")]
#[derive(Debug)]
struct AMOSWAPD(InsnT);

impl Execution for AMOSWAPD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT);
        let data = p.load_store().amo_double_word(addr, |_| { src as u64 }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00000????????????010?????0101111")]
#[derive(Debug)]
struct AMOADDW(InsnT);

impl Execution for AMOADDW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src: Wrapping<u32> = Wrapping(p.state().xreg(self.rs2() as RegT) as u32);
        let data = p.load_store().amo_word(addr, |read| {
            (src + Wrapping(read)).0
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00000????????????011?????0101111")]
#[derive(Debug)]
struct AMOADDD(InsnT);

impl Execution for AMOADDD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src: Wrapping<u64> = Wrapping(p.state().xreg(self.rs2() as RegT) as u64);
        let data = p.load_store().amo_double_word(addr, |read| {
            (src + Wrapping(read)).0
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b01100????????????010?????0101111")]
#[derive(Debug)]
struct AMOANDW(InsnT);

impl Execution for AMOANDW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u32;
        let data = p.load_store().amo_word(addr, |read| {
            src & read
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b01100????????????011?????0101111")]
#[derive(Debug)]
struct AMOADND(InsnT);

impl Execution for AMOADND {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u64;
        let data = p.load_store().amo_double_word(addr, |read| {
            src & read
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b01000????????????010?????0101111")]
#[derive(Debug)]
struct AMOORW(InsnT);

impl Execution for AMOORW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u32;
        let data = p.load_store().amo_word(addr, |read| {
            src | read
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b01000????????????011?????0101111")]
#[derive(Debug)]
struct AMOORD(InsnT);

impl Execution for AMOORD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u64;
        let data = p.load_store().amo_double_word(addr, |read| {
            src | read
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00100????????????010?????0101111")]
#[derive(Debug)]
struct AMOXORW(InsnT);

impl Execution for AMOXORW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u32;
        let data = p.load_store().amo_word(addr, |read| {
            src ^ read
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00100????????????011?????0101111")]
#[derive(Debug)]
struct AMOXORD(InsnT);

impl Execution for AMOXORD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u64;
        let data = p.load_store().amo_double_word(addr, |read| {
            src ^ read
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b10100????????????010?????0101111")]
#[derive(Debug)]
struct AMOMAXW(InsnT);

impl Execution for AMOMAXW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u32 as i32;
        let data = p.load_store().amo_word(addr, |read| {
            max(src, read as u32 as i32) as u32
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b10100????????????011?????0101111")]
#[derive(Debug)]
struct AMOMAXD(InsnT);

impl Execution for AMOMAXD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u64 as i64;
        let data = p.load_store().amo_double_word(addr, |read| {
            max(src, read as u64 as i64) as u64
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b10000????????????010?????0101111")]
#[derive(Debug)]
struct AMOMINW(InsnT);

impl Execution for AMOMINW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u32 as i32;
        let data = p.load_store().amo_word(addr, |read| {
            min(src, read as u32 as i32) as u32
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b10000????????????011?????0101111")]
#[derive(Debug)]
struct AMOMIND(InsnT);

impl Execution for AMOMIND {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u64 as i64;
        let data = p.load_store().amo_double_word(addr, |read| {
            min(src, read as u64 as i64) as u64
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b11100????????????010?????0101111")]
#[derive(Debug)]
struct AMOMAXUW(InsnT);

impl Execution for AMOMAXUW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u32;
        let data = p.load_store().amo_word(addr, |read| {
            max(src, read)
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b11100????????????011?????0101111")]
#[derive(Debug)]
struct AMOMAXUD(InsnT);

impl Execution for AMOMAXUD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u64;
        let data = p.load_store().amo_double_word(addr, |read| {
            max(src, read)
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b11000????????????010?????0101111")]
#[derive(Debug)]
struct AMOMINUW(InsnT);

impl Execution for AMOMINUW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u32;
        let data = p.load_store().amo_word(addr, |read| {
            min(src, read)
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, sext(data, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b11000????????????011?????0101111")]
#[derive(Debug)]
struct AMOMINUD(InsnT);

impl Execution for AMOMINUD {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = p.state().xreg(self.rs1() as RegT);
        let src = p.state().xreg(self.rs2() as RegT) as u64;
        let data = p.load_store().amo_double_word(addr, |read| {
            min(src, read)
        }, p.mmu())?;
        p.state().set_xreg(self.rd() as RegT, data & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}