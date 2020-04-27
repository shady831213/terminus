use crate::prelude::*;
use std::convert::TryFrom;

#[derive(Instruction)]
#[format(I)]
#[code("0b00010000001000000000000001110011")]
#[derive(Debug)]
struct SRET();

impl Execution for SRET {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('s')?;
        p.state().check_privilege_level(Privilege::S)?;
        let mcsrs = p.state().icsrs();
        let scsrs = p.state().scsrs();
        let tsr = mcsrs.mstatus().tsr();
        if tsr == 1 && *p.state().privilege() == Privilege::S {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        let spp = mcsrs.mstatus().spp();
        let spie = mcsrs.mstatus().spie();
        mcsrs.mstatus_mut().set_sie(spie);
        mcsrs.mstatus_mut().set_spie(1);
        let u_value: u8 = Privilege::U.into();
        mcsrs.mstatus_mut().set_spp(u_value as RegT);
        p.mmu().flush_tlb();
        p.fetcher().flush_icache();
        if p.state().check_extension('c').is_err() {
            let pc = (scsrs.sepc().get() >> 2) << 2;
            p.state_mut().set_pc(pc);
        } else {
            let pc = scsrs.sepc().get();
            p.state_mut().set_pc(pc);
        }
        p.state_mut().set_privilege(Privilege::try_from(spp as u8).unwrap());
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("0b0001001??????????000000001110011")]
#[derive(Debug)]
struct SFENCEVMA();

impl Execution for SFENCEVMA {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('s')?;
        p.state().check_privilege_level(Privilege::S)?;
        if *p.state().privilege() == Privilege::S && p.state().icsrs().mstatus().tvm() == 1 {
            return Err(Exception::IllegalInsn(p.state().ir()));
        }
        p.mmu().flush_tlb();
        p.fetcher().flush_icache();
        let pc = *p.state().pc() + 4;
        p.state_mut().set_pc(pc);
        Ok(())
    }
}