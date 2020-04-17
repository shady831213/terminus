use crate::processor::extensions::{NoCsr, NoStepCb};

pub struct ExtensionV{}
impl NoCsr for ExtensionV {}
impl NoStepCb for ExtensionV{}
