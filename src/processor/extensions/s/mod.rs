use crate::processor::extensions::NoStepCb;
use crate::processor::{NoCsr, ProcessorState};

mod insns;

pub struct ExtensionS {}

impl ExtensionS {
    pub fn new(_: &ProcessorState) -> ExtensionS {
        ExtensionS {}
    }
}
impl NoCsr for ExtensionS {}

impl NoStepCb for ExtensionS {}
