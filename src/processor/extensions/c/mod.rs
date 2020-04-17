use crate::processor::extensions::{NoCsr, NoStepCb};
use crate::processor::ProcessorState;

mod insns;

pub struct ExtensionC {}

impl ExtensionC {
    pub fn new(_: &ProcessorState) -> ExtensionC {
        ExtensionC {}
    }
}

impl NoCsr for ExtensionC {}

impl NoStepCb for ExtensionC{}