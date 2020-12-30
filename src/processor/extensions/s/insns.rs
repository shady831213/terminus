use crate::prelude::*;
use std::convert::TryFrom;
use crate::processor::{Processor, Privilege};
use crate::processor::trap::Exception;

#[derive(Instruction)]
#[format(I)]
#[code("32b00010000001000000000000001110011")]
#[derive(Debug)]
struct SRET();

impl Execution for SRET {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        let scsrs = p.state().scsrs()?;
        let tsr = p.state().mcsrs().mstatus().tsr();
        if tsr == 1 && *p.state().privilege() == Privilege::S {
            return Err(Exception::IllegalInsn(*p.state().ir()));
        }
        let spp = scsrs.sstatus().spp();
        let spie = scsrs.sstatus().spie();
        scsrs.sstatus_mut().set_sie(spie);
        scsrs.sstatus_mut().set_spie(1);
        let u_value: u8 = Privilege::U.into();
        scsrs.sstatus_mut().set_spp(u_value as RegT);
        if p.state().check_extension('c').is_err() {
            let pc = (scsrs.sepc().get() >> 2) << 2;
            p.state_mut().set_pc(pc);
        } else {
            let pc = scsrs.sepc().get();
            p.state_mut().set_pc(pc);
        }
        p.state_mut().set_privilege(Privilege::try_from(spp as u8).unwrap());
        p.mmu().flush_tlb();
        p.fetcher().flush_icache();
        Ok(())
    }
}

#[derive(Instruction)]
#[format(I)]
#[code("32b0001001??????????000000001110011")]
#[derive(Debug)]
struct SFENCEVMA();

impl Execution for SFENCEVMA {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('s')?;
        if *p.state().privilege() == Privilege::S && p.state().mcsrs().mstatus().tvm() == 1 {
            return Err(Exception::IllegalInsn(*p.state().ir()));
        }
        let pc = *p.state().pc() + 4;
        if self.rs1(p.state().ir()) != 0{
            let va = *p.state().xreg(self.rs1(p.state().ir()));
            p.mmu().flush_by_vpn(va >> 12);
            p.fetcher().flush_icache_by_vpn(va >> 12);
        } else {
            p.mmu().flush_tlb();
            p.fetcher().flush_icache();
        }
        p.state_mut().set_pc(pc);
        Ok(())
    }
}