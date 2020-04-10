use crate::processor::extensions::NoCsr;
use crate::processor::ProcessorCfg;

pub struct ExtensionM{}
impl ExtensionM {
    pub fn new(_: &ProcessorCfg) -> ExtensionM {
        ExtensionM{}
    }
}
impl NoCsr for ExtensionM {}