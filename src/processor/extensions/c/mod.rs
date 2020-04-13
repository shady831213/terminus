use crate::processor::extensions::NoCsr;
use crate::processor::ProcessorCfg;

mod insns;

pub struct ExtensionC {}

impl ExtensionC {
    pub fn new(_: &ProcessorCfg) -> ExtensionC {
        ExtensionC {}
    }
}

impl NoCsr for ExtensionC {}