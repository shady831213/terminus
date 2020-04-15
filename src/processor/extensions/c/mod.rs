use crate::processor::extensions::NoCsr;
use crate::processor::ProcessorState;

mod insns;

pub struct ExtensionC {}

impl ExtensionC {
    pub fn new(_: &ProcessorState) -> ExtensionC {
        ExtensionC {}
    }
}

impl NoCsr for ExtensionC {}