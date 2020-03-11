use std::collections::HashMap;
use super::extentions::Extension;
use terminus_macros::*;
use terminus_global::RegT;

mod csr;

use csr::*;

pub enum XLen {
    X32 = 32,
    X64 = 64,
}

trait CsrAccess {
    fn write(&mut self, xlen: XLen, value: RegT) {
        use XLen::*;
        match xlen {
            X64 => self.write64(value & Self::wmask64()),
            X32 => self.write32(value & Self::wmask32())
        }
    }
    fn write32(&mut self, value: RegT);
    fn write64(&mut self, value: RegT);
    fn read(&self, xlen: XLen) -> RegT {
        use XLen::*;
        match xlen {
            X64 => self.read64() & Self::rmask64(),
            X32 => self.read32() & Self::rmask32()
        }
    }
    fn read32(&self) -> RegT;
    fn read64(&self) -> RegT;
    fn mask32() -> RegT {
        0xffffffff
    }
    fn mask64() -> RegT {
        0xffffffff_ffffffff
    }
    fn wmask32() -> RegT {
        Self::mask32()
    }
    fn wmask64() -> RegT {
        Self::mask64()
    }
    fn rmask32() -> RegT {
        Self::mask32()
    }
    fn rmask64() -> RegT {
        Self::mask64()
    }
}


pub struct Processor {
    pub xreg: [RegT; 32],
    extentions: HashMap<char, Extension>,
}