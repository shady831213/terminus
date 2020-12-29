use super::{PrivilegeCode, Privilege, PrivilegeMode};
use crate::processor::ProcessorCfg;
use std::rc::Rc;
pub mod csrs;
use csrs::*;

pub struct PrivU {
    csrs: Rc<UCsrs>,
}

impl PrivU {
    pub fn new(cfg:&ProcessorCfg) -> PrivU {
        let u = PrivU{
            csrs:Rc::new(UCsrs::new(cfg.xlen.len()))
        };
        u
    }
    pub fn get_csrs(&self) -> &Rc<UCsrs> {
        &self.csrs
    }
}

impl PrivilegeCode for PrivU{
    fn code(&self) -> Privilege {
        Privilege::U
    }
}
impl PrivilegeMode for PrivU{}