use crate::processor::extensions::NoCsr;
use crate::processor::ProcessorState;

pub struct ExtensionU {}

impl ExtensionU {
    pub fn new(_: &ProcessorState) -> ExtensionU {
        let e = ExtensionU {};
        e
    }
}

impl NoCsr for ExtensionU {}