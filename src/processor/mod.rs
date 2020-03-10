use std::collections::HashMap;
use super::extentions::Extension;
use terminus_macros::*;
mod csr;
use csr::*;

pub enum XLen {
    X32 = 32,
    X64 = 64,
}

type RegT = u64;

pub struct Processor {
    pub xreg: [RegT; 32],
    extentions: HashMap<char, Extension>
}