use super::{PrivilegeCode, Privilege, PrivilegeMode};
use std::rc::Rc;
use crate::processor::ProcessorCfg;

pub mod csrs;
use csrs::*;
use crate::prelude::XLen;

pub struct PrivM {
    csrs: Rc<MCsrs>,
}

impl PrivM {
    pub fn new(cfg:&ProcessorCfg) -> PrivM {
        let m = PrivM{
            csrs:Rc::new(MCsrs::new(cfg.xlen.len()))
        };
        //no debug
        m.csrs.tselect_mut().set(0xffff_ffff_ffff_ffff);
        //mstatus
        //sd bit
        m.csrs.mstatus_mut().sd_transform({
            let csrs = m.csrs.clone();
            move |_| {
                if csrs.mstatus().fs() == 0x3 && csrs.mstatus().xs() == 0x3 {
                    1
                } else {
                    0
                }
            }
        }
        );
        m.csrs.mcycleh_mut().get_forbidden(cfg.xlen != XLen::X32);
        m.csrs.minstreth_mut().get_forbidden(cfg.xlen != XLen::X32);
        m
    }
    pub fn get_csrs(&self) -> &Rc<MCsrs> {
        &self.csrs
    }
}

impl PrivilegeCode for PrivM{
    fn code(&self) -> Privilege {
        Privilege::M
    }
}
impl PrivilegeMode for PrivM{}
