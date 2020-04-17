use crate::processor::extensions::{NoCsr, NoStepCb};
use crate::processor::ProcessorState;

mod insns;

pub struct ExtensionA {}

impl ExtensionA {
    pub fn new(_: &ProcessorState) -> ExtensionA {
        ExtensionA {}
    }
}

impl NoCsr for ExtensionA {}

impl NoStepCb for ExtensionA{}