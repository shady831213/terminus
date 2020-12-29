use crate::processor::extensions::NoStepCb;
use crate::processor::{ProcessorState,NoCsr};

mod insns;

pub struct ExtensionI {
}

impl ExtensionI {
    pub fn new(_: &ProcessorState) -> ExtensionI {
        ExtensionI{}
    }
}
impl NoCsr for ExtensionI{}
impl NoStepCb for ExtensionI{}

