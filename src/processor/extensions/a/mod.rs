use crate::processor::extensions::{NoCsr, HasStepCb};
use crate::processor::{ProcessorState, Processor};
use std::cell::RefCell;
use terminus_global::RegT;

mod insns;

struct LCReservation {
    valid:bool,
    addr: RegT,
    len: u64,
    timestamp: u64,
}

pub struct ExtensionA {
    lc_res: RefCell<LCReservation>
}

impl ExtensionA {
    pub fn new(_: &ProcessorState) -> ExtensionA
    {
        ExtensionA {
            lc_res: RefCell::new(LCReservation{valid:false, addr: 0, len:0, timestamp:0})
        }
    }

}

impl NoCsr for ExtensionA {}

impl HasStepCb for ExtensionA {
    fn step_cb(&self, p: &Processor) {
        let mut lc_res = self.lc_res.borrow_mut();
        if lc_res.valid {
            if p.state().insns_cnt() > lc_res.timestamp + 16 {
                lc_res.valid = false;
                p.load_store().release()
            }
        }
    }
}