use crate::prelude::*;
use crate::processor::{Processor, Privilege};
use crate::processor::trap::Exception;

#[derive(Instruction)]
#[format(I)]
#[code("32b00010000001000000000000001110011")]
#[derive(Debug)]
struct SRET();

impl Execution for SRET {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception> {
        p.state().check_extension('s')?;
        let tsr = p.state().priv_m().mstatus().tsr();
        if tsr == 1 && *p.state().privilege() == Privilege::S {
            return Err(Exception::IllegalInsn(*p.state().ir()));
        }
        p.state_mut().trap_return(&Privilege::S);
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
        if *p.state().privilege() == Privilege::S && p.state().priv_m().mstatus().tvm() == 1 {
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