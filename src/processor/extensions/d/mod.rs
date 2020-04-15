use crate::processor::extensions::NoCsr;
use crate::processor::ProcessorState;

// mod insns;
pub struct ExtensionD{}
impl ExtensionD {
    pub fn new(_: &ProcessorState) -> ExtensionD {
        ExtensionD{}
    }
}
impl NoCsr for ExtensionD {}