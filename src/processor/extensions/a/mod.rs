use crate::processor::extensions::{NoCsr, HasStepCb};
use crate::processor::{ProcessorState, Processor};
use std::cell::{RefCell, Ref};
use std::ops::Deref;
use terminus_global::RegT;

mod insns;

struct LCReservation {
    addr: RegT,
    len: u64,
    timestamp: u64,
}

pub struct ExtensionA {
    lc_res: RefCell<Option<LCReservation>>
}

impl ExtensionA {
    pub fn new(_: &ProcessorState) -> ExtensionA
    {
        ExtensionA {
            lc_res: RefCell::new(None)
        }
    }

    fn set_lc_res(&self, addr: RegT, len: u64, timestamp: u64) {
        *self.lc_res.borrow_mut() = Some(LCReservation {
            addr,
            len,
            timestamp,
        })
    }

    fn clr_lc_res(&self) {
        *self.lc_res.borrow_mut() = None;
    }

    fn lc_res(&self) -> Ref<'_, Option<LCReservation>> {
        self.lc_res.borrow()
    }
}

impl NoCsr for ExtensionA {}

impl HasStepCb for ExtensionA {
    fn step_cb(&self, p: &Processor) {
        if let Some(lc_res) = self.lc_res().deref() {
            if p.state().insns_cnt() > lc_res.timestamp + 16 {
                self.clr_lc_res();
                p.load_store().release()
            }
        }
    }
}