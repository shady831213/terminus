use crate::processor::ProcessorCfg;
use std::ops::Deref;
use std::rc::Rc;

pub mod csrs;
use super::PrivM;
use csrs::*;

pub struct PrivU {
    csrs: Rc<UCsrs>,
}

impl PrivU {
    pub fn new(cfg: &ProcessorCfg, m: &PrivM) -> PrivU {
        let u = PrivU {
            csrs: Rc::new(UCsrs::new(cfg.xlen.len())),
        };
        u.csrs.instret_mut().instret_transform({
            let csrs = (*m).clone();
            move |_| csrs.minstret().get()
        });
        u.csrs.instreth_mut().instret_transform({
            let csrs = (*m).clone();
            move |_| csrs.minstreth().get()
        });
        u
    }
}

impl Deref for PrivU {
    type Target = Rc<UCsrs>;
    fn deref(&self) -> &Self::Target {
        &self.csrs
    }
}
