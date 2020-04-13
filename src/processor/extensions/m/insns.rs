use terminus_global::*;
use terminus_macros::*;
use terminus_proc_macros::Instruction;
use crate::processor::{Processor, Privilege, PrivilegeLevel};
use crate::processor::trap::Exception;
use crate::processor::insn::*;
use crate::processor::decode::*;
use crate::linkme::*;
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
struct MUL(InsnT);

impl Execution for MUL {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs1() as RegT));
        let rs2: Wrapping<RegT> = Wrapping(p.state().xreg(self.rs2() as RegT));
        p.state().set_xreg(self.rd() as RegT, (rs1 * rs2).0 & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????001?????0110011")]
#[derive(Debug)]
struct MULH(InsnT);

impl Execution for MULH {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1 = BigInt::from(sext(p.state().xreg(self.rs1() as RegT), p.state().config().xlen.len()) as SRegT);
        let rs2 = BigInt::from(sext(p.state().xreg(self.rs2() as RegT), p.state().config().xlen.len()) as SRegT);
        p.state().set_xreg(self.rd() as RegT, bigint_to_reg((rs1 * rs2) >> p.state().config().xlen.len(), p.state().config().xlen.size()) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????010?????0110011")]
#[derive(Debug)]
struct MULHSU(InsnT);

impl Execution for MULHSU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1 = BigInt::from(sext(p.state().xreg(self.rs1() as RegT), p.state().config().xlen.len()) as SRegT);
        let rs2 = BigInt::from(p.state().xreg(self.rs2() as RegT) & p.state().config().xlen.mask());
        let product = (rs1 * rs2) >> p.state().config().xlen.len();
        p.state().set_xreg(self.rd() as RegT, bigint_to_reg(product, p.state().config().xlen.size()) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????011?????0110011")]
#[derive(Debug)]
struct MULHU(InsnT);

impl Execution for MULHU {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        let rs1 = BigInt::from(p.state().xreg(self.rs1() as RegT) & p.state().config().xlen.mask());
        let rs2 = BigInt::from(p.state().xreg(self.rs2() as RegT) & p.state().config().xlen.mask());
        p.state().set_xreg(self.rd() as RegT, bigint_to_reg((rs1 * rs2) >> p.state().config().xlen.len(), p.state().config().xlen.size()) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}

#[derive(Instruction)]
#[format(R)]
#[code("0b0000001??????????000?????0111011")]
#[derive(Debug)]
struct MULW(InsnT);

impl Execution for MULW {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('m')?;
        p.state().check_xlen(XLen::X64)?;
        let rs1: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs1() as RegT), 32));
        let rs2: Wrapping<RegT> = Wrapping(sext(p.state().xreg(self.rs2() as RegT), 32));
        p.state().set_xreg(self.rd() as RegT, sext((rs1 * rs2).0, 32) & p.state().config().xlen.mask());
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}