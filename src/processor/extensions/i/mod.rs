use crate::processor::extensions::NoStepCb;
use crate::processor::{NoCsr, ProcessorState};

mod insns;

pub struct ExtensionI {}

impl ExtensionI {
    pub fn new(_: &ProcessorState) -> ExtensionI {
        ExtensionI {}
    }
}
impl NoCsr for ExtensionI {}
impl NoStepCb for ExtensionI {}
