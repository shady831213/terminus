use crate::processor::extensions::{NoCsr, NoStepCb};
use crate::processor::ProcessorState;

mod insns;

pub struct ExtensionM{}
impl ExtensionM {
    pub fn new(_: &ProcessorState) -> ExtensionM {
        ExtensionM{}
    }
}
impl NoCsr for ExtensionM {}

impl NoStepCb for ExtensionM{}
