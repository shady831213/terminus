extern crate num;

use crate::prelude::*;
use std::num::Wrapping;
use num::BigInt;
use num::bigint::Sign;
use std::cmp::min;

fn bigint_to_reg(b: BigInt, size: usize) -> RegT {
    let v = b.to_signed_bytes_le();
    let mut value = v[..min(size, v.len())].iter().enumerate()
        .map(|(i, v)| { (*v as RegT) << ((i as RegT) << 3) })
        .fold(0 as RegT, |acc, v| { acc | v });
    //padding
    if v.len() < size {
        for p in v.len()..size {
            value.set_bit_range((p << 3) + 7, p << 3, if b.sign() == Sign::Minus { 0xff } else { 0x0 })
        }
    }
    value
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????000?????0110011")]
#[derive(Debug)]
struct MUL();

impl Execution for MUL {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs2(p.state().ir())));
        let rd = self.rd(p.state().ir());
        let value = (rs1 * rs2).0 & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????001?????0110011")]
#[derive(Debug)]
struct MULH();

impl Execution for MULH {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1 = BigInt::from(sext(*p.state().xreg(self.rs1(p.state().ir())), p.state().config().xlen.len()) as SRegT);
        let rs2 = BigInt::from(sext(*p.state().xreg(self.rs2(p.state().ir())), p.state().config().xlen.len()) as SRegT);
        let rd = self.rd(p.state().ir());
        let value = bigint_to_reg((rs1 * rs2) >> p.state().config().xlen.len(), p.state().config().xlen.size()) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????010?????0110011")]
#[derive(Debug)]
struct MULHSU();

impl Execution for MULHSU {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1 = BigInt::from(sext(*p.state().xreg(self.rs1(p.state().ir())), p.state().config().xlen.len()) as SRegT);
        let rs2 = BigInt::from(*p.state().xreg(self.rs2(p.state().ir())) & p.state().config().xlen.mask());
        let product = (rs1 * rs2) >> p.state().config().xlen.len();
        let rd = self.rd(p.state().ir());
        let value = bigint_to_reg(product, p.state().config().xlen.size()) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????011?????0110011")]
#[derive(Debug)]
struct MULHU();

impl Execution for MULHU {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1 = BigInt::from(*p.state().xreg(self.rs1(p.state().ir())) & p.state().config().xlen.mask());
        let rs2 = BigInt::from(*p.state().xreg(self.rs2(p.state().ir())) & p.state().config().xlen.mask());
        let rd = self.rd(p.state().ir());
        let value = bigint_to_reg((rs1 * rs2) >> p.state().config().xlen.len(), p.state().config().xlen.size()) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????000?????0111011")]
#[derive(Debug)]
struct MULW();

impl Execution for MULW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(*p.state().xreg(self.rs1(p.state().ir())), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(*p.state().xreg(self.rs2(p.state().ir())), 32));
        let rd = self.rd(p.state().ir());
        let value = sext((rs1 * rs2).0, 32) & p.state().config().xlen.mask();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_xreg(rd, value);
        p.state_mut().set_pc(pc);
        Ok(())
    }
}


#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????100?????0110011")]
#[derive(Debug)]
struct DIV();

impl Execution for DIV {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1: Wrapping<SRegT> = Wrapping(sext(*p.state().xreg(self.rs1(p.state().ir())), p.state().config().xlen.len()) as SRegT);
        let rs2: Wrapping<SRegT> = Wrapping(sext(*p.state().xreg(self.rs2(p.state().ir())), p.state().config().xlen.len()) as SRegT);
        if rs2 == Wrapping(0 as SRegT) {
            let rd = self.rd(p.state().ir());
            let value = (-1 as SRegT) as RegT & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = (rs1 / rs2).0 as RegT & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????101?????0110011")]
#[derive(Debug)]
struct DIVU();

impl Execution for DIVU {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs2(p.state().ir())));
        if rs2 == Wrapping(0 as RegT) {
            let rd = self.rd(p.state().ir());
            let value = (-1 as SRegT) as RegT & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = (rs1 / rs2).0 & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????100?????0111011")]
#[derive(Debug)]
struct DIVW();

impl Execution for DIVW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<SRegT> = Wrapping(sext(*p.state().xreg(self.rs1(p.state().ir())), 32) as SRegT);
        let rs2: Wrapping<SRegT> = Wrapping(sext(*p.state().xreg(self.rs2(p.state().ir())), 32) as SRegT);
        if rs2 == Wrapping(0 as SRegT) {
            let rd = self.rd(p.state().ir());
            let value = (-1 as SRegT) as RegT & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = sext((rs1 / rs2).0 as RegT, 32) & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????101?????0111011")]
#[derive(Debug)]
struct DIVUW();

impl Execution for DIVUW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        p.state().check_xlen(XLen::X64)?;
        let rs1_u: RegT = *p.state().xreg(self.rs1(p.state().ir())) & 0xffff_ffff;
        let rs1: Wrapping<RegT> = Wrapping(rs1_u);
        let rs2_u: RegT = *p.state().xreg(self.rs2(p.state().ir())) & 0xffff_ffff;
        let rs2: Wrapping<RegT> = Wrapping(rs2_u);
        if rs2 == Wrapping(0 as RegT) {
            let rd = self.rd(p.state().ir());
            let value = (-1 as SRegT) as RegT & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = sext((rs1 / rs2).0 as RegT, 32) & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????110?????0110011")]
#[derive(Debug)]
struct REM();

impl Execution for REM {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1: Wrapping<SRegT> = Wrapping(sext(*p.state().xreg(self.rs1(p.state().ir())), p.state().config().xlen.len()) as SRegT);
        let rs2: Wrapping<SRegT> = Wrapping(sext(*p.state().xreg(self.rs2(p.state().ir())), p.state().config().xlen.len()) as SRegT);
        if rs2 == Wrapping(0 as SRegT) {
            let rd = self.rd(p.state().ir());
            let value = rs1.0 as RegT & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = (rs1 % rs2).0 as RegT & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????111?????0110011")]
#[derive(Debug)]
struct REMU();

impl Execution for REMU {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs1(p.state().ir())));
        let rs2: Wrapping<RegT> = Wrapping(*p.state().xreg(self.rs2(p.state().ir())));
        if rs2 == Wrapping(0 as RegT) {
            let rd = self.rd(p.state().ir());
            let value = rs1.0 & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = (rs1 % rs2).0 & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????110?????0111011")]
#[derive(Debug)]
struct REMW();

impl Execution for REMW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<SRegT> = Wrapping(sext(*p.state().xreg(self.rs1(p.state().ir())), 32) as SRegT);
        let rs2: Wrapping<SRegT> = Wrapping(sext(*p.state().xreg(self.rs2(p.state().ir())), 32) as SRegT);
        if rs2 == Wrapping(0 as SRegT) {
            let rd = self.rd(p.state().ir());
            let value = sext(rs1.0 as RegT, 32) & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = sext((rs1 % rs2).0 as RegT, 32) & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????111?????0111011")]
#[derive(Debug)]
struct REMUW();

impl Execution for REMUW {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        p.state().check_xlen(XLen::X64)?;
        let rs1_u: RegT = *p.state().xreg(self.rs1(p.state().ir())) & 0xffff_ffff;
        let rs1: Wrapping<RegT> = Wrapping(rs1_u);
        let rs2_u: RegT = *p.state().xreg(self.rs2(p.state().ir())) & 0xffff_ffff;
        let rs2: Wrapping<RegT> = Wrapping(rs2_u);
        if rs2 == Wrapping(0 as RegT) {
            let rd = self.rd(p.state().ir());
            let value = sext(rs1.0, 32) & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        } else {
            let rd = self.rd(p.state().ir());
            let value = sext((rs1 % rs2).0 as RegT, 32) & p.state().config().xlen.mask();
            p.state_mut().set_xreg(rd, value);
        }
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}