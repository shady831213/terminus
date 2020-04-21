use terminus_global::*;
use std::rc::Rc;
use std::any::Any;
use crate::processor::extensions::{HasCsr, NoStepCb};
use crate::processor::extensions::s::csrs::*;

mod insns;
pub mod csrs;

use csrs::ICsrs;
use crate::processor::{PrivilegeLevel, Privilege, ProcessorState};
use std::ops::Deref;

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

        //deleg counter
        e.csrs.instret_mut().instret_transform({
            let count = state.insns_cnt.clone();
            move |_| {
                *count.deref().borrow() as RegT
            }
        }
        );
        e.csrs.instreth_mut().instret_transform({
            let count = state.insns_cnt.clone();
            move |_| {
                (*count.deref().borrow() >> 32) as RegT
            }
        }
        );
        e.csrs.minstret_mut().instret_transform({
            let count = state.insns_cnt.clone();
            move |_| {
                *count.deref().borrow() as RegT
            }
        }
        );
        e.csrs.minstreth_mut().instret_transform({
            let count = state.insns_cnt.clone();
            move |_| {
                (*count.deref().borrow() >> 32) as RegT
            }
        }
        );
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
    fn csr_read(&self, state: &ProcessorState, addr: RegT) -> Option<RegT> {
        let addr_high = addr & 0xff0;
        if (addr_high == 0xc80 || addr_high == 0xc90 || addr_high == 0xb80 || addr_high == 0xb90) && state.config().xlen != XLen::X32 {
            return None
        }
        if addr_high == 0xc80 || addr_high == 0xc90 || addr_high == 0xc00 || addr_high == 0xc10 {
            match state.privilege() {
                Privilege::M => {}
                Privilege::S => {
                    if self.csrs.mcounteren().get() & ((1 as RegT) << (addr & 0x1f)) == 0 {
                        return None
                    }
                }
                Privilege::U => {
                    if self.csrs.mcounteren().get() & ((1 as RegT) << (addr & 0x1f)) == 0 {
                        return None
                    }
                    if let Ok(scsrs) = state.csrs::<SCsrs>() {
                        if scsrs.scounteren().get() & ((1 as RegT) << (addr & 0x1f)) == 0 {
                            return None
                        }
                    }
                }
            }
        }
        self.csrs.read(addr)
    }
}

impl NoStepCb for ExtensionI{}

