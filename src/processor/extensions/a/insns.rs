use crate::prelude::*;
use crate::processor::extensions::a::ExtensionA;
use std::rc::Rc;
use crate::processor::extensions::Extension;
use std::num::Wrapping;
use std::cmp::{min, max};

pub trait LRSCInsn: InstructionImp {
    fn get_a_ext(&self, p: &Processor) -> Result<Rc<ExtensionA>, Exception> {
        p.state().check_extension('a')?;
        if let Extension::A(ref a) = p.state().get_extension('a') {
            Ok(a.clone())
        } else {
            Err(Exception::IllegalInsn(p.state().ir()))
        }
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00010??00000?????010?????0101111")]
#[derive(Debug)]
struct LRW();

impl LRSCInsn for LRW {}

impl Execution for LRW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let a = self.get_a_ext(p)?;
        let mut lc_res = a.lc_res.borrow_mut();
        lc_res.valid = false;
        p.load_store().release(p.state());
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let success = p.load_store().acquire(p.state(), addr, 4, p.mmu())?;
        let data = p.load_store().load_word(p.state(), addr, p.mmu())?;
        if success {
            lc_res.valid = true;
            lc_res.addr = addr;
            lc_res.len = 4;
            lc_res.timestamp = *p.state().insns_cnt().borrow();
        }
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
#[code("0b00010??00000?????011?????0101111")]
#[derive(Debug)]
struct LRD();

impl LRSCInsn for LRD {}

impl Execution for LRD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let state = p.state();
        state.check_xlen(XLen::X64)?;
        let a = self.get_a_ext(p)?;
        let mut lc_res = a.lc_res.borrow_mut();
        lc_res.valid = false;
        p.load_store().release(state);
        let addr = *state.xreg(self.rs1(state.ir()));
        let success = p.load_store().acquire(state, addr, 8, p.mmu())?;
        let data = p.load_store().load_double_word(state, addr, p.mmu())?;
        if success {
            lc_res.valid = true;
            lc_res.addr = addr;
            lc_res.len = 8;
            lc_res.timestamp = *state.insns_cnt().borrow();
        }
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        let state = p.state_mut();
        state.set_xreg(rd, value);
        state.set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00011????????????010?????0101111")]
#[derive(Debug)]
struct SCW();

impl LRSCInsn for SCW {}

impl Execution for SCW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let a = self.get_a_ext(p)?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let data = *p.state().xreg(self.rs2(p.state().ir()));
        let mut lc_res = a.lc_res.borrow_mut();
        let success = if lc_res.valid {
            if addr != lc_res.addr || lc_res.len != 4 {
                false
            } else {
                p.load_store().check_lock(p.state(), addr, 4, p.mmu())?
            }
        } else {
            false
        };
        if success {
            p.load_store().store_word(p.state(), addr, data, p.mmu())?
        }
        lc_res.valid = false;
        p.load_store().release(p.state());
        let rd = self.rd(p.state().ir());
        let value = (!success) as RegT;
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00011????????????011?????0101111")]
#[derive(Debug)]
struct SCD();

impl LRSCInsn for SCD {}

impl Execution for SCD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        let a = self.get_a_ext(p)?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let data = *p.state().xreg(self.rs2(p.state().ir()));
        let mut lc_res = a.lc_res.borrow_mut();
        let success = if lc_res.valid {
            if addr != lc_res.addr || lc_res.len != 8 {
                false
            } else {
                p.load_store().check_lock(p.state(), addr, 8, p.mmu())?
            }
        } else {
            false
        };
        if success {
            p.load_store().store_double_word(p.state(), addr, data, p.mmu())?
        }
        lc_res.valid = false;
        p.load_store().release(p.state());
        let rd = self.rd(p.state().ir());
        let value = (!success) as RegT;
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00001????????????010?????0101111")]
#[derive(Debug)]
struct AMOSWAPW();

impl Execution for AMOSWAPW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir()));
        let data = p.load_store().amo_word(p.state(), addr, |_| { src as u32 }, p.mmu())?;
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
#[code("0b00001????????????011?????0101111")]
#[derive(Debug)]
struct AMOSWAPD();

impl Execution for AMOSWAPD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir()));
        let data = p.load_store().amo_double_word(p.state(), addr, |_| { src as u64 }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00000????????????010?????0101111")]
#[derive(Debug)]
struct AMOADDW();

impl Execution for AMOADDW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src: Wrapping<u32> = Wrapping(*p.state().xreg(self.rs2(p.state().ir())) as u32);
        let data = p.load_store().amo_word(p.state(), addr, |read| {
            (src + Wrapping(read)).0
        }, p.mmu())?;
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
#[code("0b00000????????????011?????0101111")]
#[derive(Debug)]
struct AMOADDD();

impl Execution for AMOADDD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src: Wrapping<u64> = Wrapping(*p.state().xreg(self.rs2(p.state().ir())) as u64);
        let data = p.load_store().amo_double_word(p.state(), addr, |read| {
            (src + Wrapping(read)).0
        }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b01100????????????010?????0101111")]
#[derive(Debug)]
struct AMOANDW();

impl Execution for AMOANDW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u32;
        let data = p.load_store().amo_word(p.state(), addr, |read| {
            src & read
        }, p.mmu())?;
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
#[code("0b01100????????????011?????0101111")]
#[derive(Debug)]
struct AMOADND();

impl Execution for AMOADND {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u64;
        let data = p.load_store().amo_double_word(p.state(), addr, |read| {
            src & read
        }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b01000????????????010?????0101111")]
#[derive(Debug)]
struct AMOORW();

impl Execution for AMOORW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u32;
        let data = p.load_store().amo_word(p.state(), addr, |read| {
            src | read
        }, p.mmu())?;
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
#[code("0b01000????????????011?????0101111")]
#[derive(Debug)]
struct AMOORD();

impl Execution for AMOORD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u64;
        let data = p.load_store().amo_double_word(p.state(), addr, |read| {
            src | read
        }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b00100????????????010?????0101111")]
#[derive(Debug)]
struct AMOXORW();

impl Execution for AMOXORW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u32;
        let data = p.load_store().amo_word(p.state(), addr, |read| {
            src ^ read
        }, p.mmu())?;
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
#[code("0b00100????????????011?????0101111")]
#[derive(Debug)]
struct AMOXORD();

impl Execution for AMOXORD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u64;
        let data = p.load_store().amo_double_word(p.state(), addr, |read| {
            src ^ read
        }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b10100????????????010?????0101111")]
#[derive(Debug)]
struct AMOMAXW();

impl Execution for AMOMAXW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u32 as i32;
        let data = p.load_store().amo_word(p.state(), addr, |read| {
            max(src, read as u32 as i32) as u32
        }, p.mmu())?;
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
#[code("0b10100????????????011?????0101111")]
#[derive(Debug)]
struct AMOMAXD();

impl Execution for AMOMAXD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u64 as i64;
        let data = p.load_store().amo_double_word(p.state(), addr, |read| {
            max(src, read as u64 as i64) as u64
        }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b10000????????????010?????0101111")]
#[derive(Debug)]
struct AMOMINW();

impl Execution for AMOMINW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u32 as i32;
        let data = p.load_store().amo_word(p.state(), addr, |read| {
            min(src, read as u32 as i32) as u32
        }, p.mmu())?;
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
#[code("0b10000????????????011?????0101111")]
#[derive(Debug)]
struct AMOMIND();

impl Execution for AMOMIND {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u64 as i64;
        let data = p.load_store().amo_double_word(p.state(), addr, |read| {
            min(src, read as u64 as i64) as u64
        }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b11100????????????010?????0101111")]
#[derive(Debug)]
struct AMOMAXUW();

impl Execution for AMOMAXUW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u32;
        let data = p.load_store().amo_word(p.state(), addr, |read| {
            max(src, read)
        }, p.mmu())?;
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
#[code("0b11100????????????011?????0101111")]
#[derive(Debug)]
struct AMOMAXUD();

impl Execution for AMOMAXUD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u64;
        let data = p.load_store().amo_double_word(p.state(), addr, |read| {
            max(src, read)
        }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b11000????????????010?????0101111")]
#[derive(Debug)]
struct AMOMINUW();

impl Execution for AMOMINUW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u32;
        let data = p.load_store().amo_word(p.state(), addr, |read| {
            min(src, read)
        }, p.mmu())?;
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
#[code("0b11000????????????011?????0101111")]
#[derive(Debug)]
struct AMOMINUD();

impl Execution for AMOMINUD {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_xlen(XLen::X64)?;
        p.state().check_extension('a')?;
        let addr = *p.state().xreg(self.rs1(p.state().ir()));
        let src = *p.state().xreg(self.rs2(p.state().ir())) as u64;
        let data = p.load_store().amo_double_word(p.state(), addr, |read| {
            min(src, read)
        }, p.mmu())?;
        let rd = self.rd(p.state().ir());
        let value = data & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}