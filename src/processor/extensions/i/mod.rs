use terminus_global::*;
use std::rc::Rc;
use std::any::Any;
use crate::processor::extensions::HasCsr;

mod insns;
pub mod csrs;

use csrs::ICsrs;
use crate::processor::{PrivilegeLevel, Privilege, ProcessorState};

pub struct ExtensionI {
    csrs: Rc<ICsrs>,
}

impl ExtensionI {
    pub fn new(state: &ProcessorState) -> ExtensionI {
        let cfg = state.config();
        let e = ExtensionI {
            csrs: Rc::new(ICsrs::new(cfg.xlen))
        };
        //no debug
        e.csrs.tselect_mut().set(0xffff_ffff_ffff_ffff);
        //mstatus
        //sd bit
        e.csrs.mstatus_mut().sd_transform({
            let csrs = e.csrs.clone();
            move |_| {
                if csrs.mstatus().fs() == 0x3 && csrs.mstatus().xs() == 0x3 {
                    1
                } else {
                    0
                }
            }
        }
        );
        //will be overrided if 's' implemented
        e.csrs.mstatus_mut().set_tvm_transform(|_| { 0 });
        e.csrs.mstatus_mut().set_tsr_transform(|_| { 0 });
        //xlen config
        match cfg.xlen {
            XLen::X32 => {
                e.csrs.misa_mut().set_mxl(1);
            }
            XLen::X64 => {
                e.csrs.misa_mut().set_mxl(2);
                e.csrs.mstatus_mut().set_uxl(2);
                e.csrs.mstatus_mut().set_sxl(2);
            }
        }

        //privilege_level config
        match cfg.privilege_level() {
            PrivilegeLevel::MSU => {}
            PrivilegeLevel::MU => {
                e.csrs.mstatus_mut().set_mpp_transform(|mpp| {
                    if mpp != 0 {
                        let m: u8 = Privilege::M.into();
                        m as RegT
                    } else {
                        0
                    }
                });
            }
            PrivilegeLevel::M => {
                let m: u8 = Privilege::M.into();
                e.csrs.mstatus_mut().set_mpp(m as RegT);
                e.csrs.mstatus_mut().set_mpp_transform(move |_| {
                    m as RegT
                });
                e.csrs.mstatus_mut().set_tw_transform(|_| { 0 });
            }
        }

        e
    }
}

impl HasCsr for ExtensionI {
    fn csrs(&self) -> Option<Rc<dyn Any>> {
        Some(self.csrs.clone() as Rc<dyn Any>)
    }
    fn csr_write(&self, _: &ProcessorState, addr: RegT, value: RegT) -> Option<()> {
        self.csrs.write(addr, value)
    }
    fn csr_read(&self, _: &ProcessorState, addr: RegT) -> Option<RegT> {
        self.csrs.read(addr)
    }
}

