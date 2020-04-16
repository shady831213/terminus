use terminus_spaceport::memory::region::{U16Access, U32Access};
use terminus_spaceport::memory::region;
use crate::processor::{ProcessorState, Instruction};
use std::rc::Rc;
use terminus_global::{RegT, InsnT};
use crate::processor::mmu::{Mmu, MmuOpt};
use crate::processor::trap::Exception;
use crate::processor::decode::*;
use std::sync::Arc;
use crate::system::Bus;
use std::ops::Deref;

pub struct Fetcher {
    p: Rc<ProcessorState>,
    bus: Arc<Bus>,
}

impl Fetcher {
    pub fn new(p: &Rc<ProcessorState>, bus: &Arc<Bus>) -> Fetcher {
        Fetcher {
            p: p.clone(),
            bus: bus.clone(),
        }
    }

    pub fn fetch(&self, pc: RegT, mmu: &Mmu) -> Result<Instruction, Exception> {
        let code = {
            //expect compress, if is not support, raise illegeInst exception later
            if pc.trailing_zeros() == 1 {
                let pa = mmu.translate(pc, 2, MmuOpt::Fetch)?;
                match U16Access::read(self.bus.deref(), pa) {
                    Ok(data) => {
                        //expect compress, if is not support, raise illegeInst exception later
                        if data & 0x3 != 0x3 {
                            data as u16 as InsnT
                        } else {
                            let pa_high = mmu.translate(pc + 2, 2, MmuOpt::Fetch)?;
                            let data_high = match U16Access::read(self.bus.deref(), pa_high) {
                                Ok(data_h) => {
                                    (data_h as InsnT) << 16
                                }
                                Err(e) => match e {
                                    region::Error::AccessErr(_, _) => return Err(Exception::FetchAccess(pc)),
                                    region::Error::Misaligned(_) => return Err(Exception::FetchMisaligned(pc))
                                }
                            };
                            data as u16 as InsnT | data_high
                        }
                    }
                    Err(e) => match e {
                        region::Error::AccessErr(_, _) => return Err(Exception::FetchAccess(pc)),
                        region::Error::Misaligned(_) => return Err(Exception::FetchMisaligned(pc))
                    }
                }
            } else {
                let pa = mmu.translate(pc, 4, MmuOpt::Fetch)?;
                match U32Access::read(self.bus.deref(), pa) {
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