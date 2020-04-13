use terminus_global::*;
use terminus_macros::*;
use terminus_proc_macros::Instruction;
use crate::processor::{Processor, Privilege};
use crate::processor::trap::Exception;
use crate::processor::insn::*;
use crate::processor::decode::*;
use crate::linkme::*;
use crate::processor::extensions::i::csrs::*;
use crate::processor::extensions::s::csrs::*;
use std::convert::TryFrom;

#[derive(Instruction)]
#[format(I)]
#[code("0b00010000001000000000000001110011")]
#[derive(Debug)]
struct SRET(InsnT);

impl Execution for SRET {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('s')?;
        p.state().check_privilege_level(Privilege::S)?;
        let mcsrs = p.state().csrs::<ICsrs>().unwrap();
        let scsrs = p.state().csrs::<SCsrs>().unwrap();
        let tsr = mcsrs.mstatus().tsr();
        if tsr == 1 && p.state().privilege() == Privilege::S {
            return Err(Exception::IllegalInsn(self.ir()))
        }
        let spp = mcsrs.mstatus().spp();
        let spie = mcsrs.mstatus().spie();
        mcsrs.mstatus_mut().set_sie(spie);
        mcsrs.mstatus_mut().set_spie(1);
        let u_value: u8 = Privilege::U.into();
        mcsrs.mstatus_mut().set_spp(u_value as RegT);
        p.state().set_privilege(Privilege::try_from(spp as u8).unwrap());
        p.state().set_pc(scsrs.sepc().get());
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0001001??????????000000001110011")]
#[derive(Debug)]
struct SFENCEVMA(InsnT);

impl Execution for SFENCEVMA {
    fn execute(&self, p: &Processor) -> Result<(), Exception> {
        p.state().check_extension('s')?;
        p.state().check_privilege_level(Privilege::S)?;
        if p.state().privilege() == Privilege::S && p.state().csrs::<ICsrs>().unwrap().mstatus().tvm() == 1 {
            return Err(Exception::IllegalInsn(self.ir()));
        }
        //fixme:no cache in fetcher, load_store and mmu for now
        p.state().set_pc(p.state().pc() + 4);
        Ok(())
    }
}