use crate::processor::ProcessorCfg;
use std::rc::Rc;
pub mod csrs;
use csrs::*;
use super::MCsrs;

pub struct PrivU {
    csrs: Rc<UCsrs>,
}

impl PrivU {
    pub fn new(cfg:&ProcessorCfg, mcsrs:&Rc<MCsrs>) -> PrivU {
        let u = PrivU{
            csrs:Rc::new(UCsrs::new(cfg.xlen.len()))
        };
        u.csrs.instret_mut().instret_transform({
            let csrs = mcsrs.clone();
            move |_| {
                csrs.minstret().get()
            }
        }
        );
        u.csrs.instreth_mut().instret_transform({
            let csrs = mcsrs.clone();
            move |_| {
                csrs.minstreth().get()
            }
        }
        );
        u
    }
    pub fn get_csrs(&self) -> &Rc<UCsrs> {
        &self.csrs
    }
}
