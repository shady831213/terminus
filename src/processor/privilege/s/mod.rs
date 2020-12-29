use super::{PrivilegeCode, Privilege, PrivilegeMode};
use std::rc::Rc;
use crate::processor::ProcessorCfg;
pub mod csrs;
use csrs::*;

pub struct PrivS {
    csrs: Rc<SCsrs>,
}
impl PrivS {
    pub fn new(cfg: &ProcessorCfg) -> PrivS {
        let s = PrivS {
            csrs:Rc::new(SCsrs::new(cfg.xlen.len()))
        };
        s
    }
    pub fn get_csrs(&self) -> &Rc<SCsrs> {
        &self.csrs
    }
}

impl PrivilegeCode for PrivS{
    fn code(&self) -> Privilege {
        Privilege::S
    }
}
impl PrivilegeMode for PrivS{}