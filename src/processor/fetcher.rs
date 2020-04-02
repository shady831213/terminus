use terminus_spaceport::memory::region::{U16Access, U32Access};
use terminus_spaceport::memory::region;
use crate::processor::{ProcessorState, Instruction};
use std::rc::Rc;
use terminus_global::{RegT, InsnT};
use crate::processor::mmu::{Mmu, MmuOpt};
use crate::processor::execption::Exception;
use crate::processor::decode::*;

pub struct Fetcher {
    p: Rc<ProcessorState>,
}

impl Fetcher {
    pub fn new(p: &Rc<ProcessorState>) -> Fetcher {
        Fetcher {
            p: p.clone(),
        }
    }

    pub fn fetch(&self, pc: RegT, mmu: &Mmu) -> Result<Instruction, Exception> {
        if pc.trailing_zeros() == 0 {
            return Err(Exception::FetchMisaligned(pc));
        }
        let code = {
            //expect compress, if is not support, raise illegeInst exception later
            if pc.trailing_zeros() == 1 {
                let pa = mmu.translate(pc, 2, MmuOpt::Fetch)?;
                match U16Access::read(&self.p.bus, pa) {
                    Ok(data) => data as InsnT,
                    Err(e) => match e {
                        region::Error::AccessErr(_, _) => return Err(Exception::FetchAccess(pc)),
                        region::Error::Misaligned(_) => return Err(Exception::FetchMisaligned(pc))
                    }
                }
            } else {
                let pa = mmu.translate(pc, 4, MmuOpt::Fetch)?;
                match U32Access::read(&self.p.bus, pa) {
                    Ok(data) => {
                        //expect compress, if is not support, raise illegeInst exception later
                        if data & 0x3 != 0x3 {
                            data as u16 as InsnT
                        } else {
                            data as InsnT
                        }
                    }
                    Err(e) => match e {
                        region::Error::AccessErr(_, _) => return Err(Exception::FetchAccess(pc)),
                        region::Error::Misaligned(_) => return Err(Exception::FetchMisaligned(pc))
                    }
                }
            }
        };
        GDECODER.decode(code)
    }
}