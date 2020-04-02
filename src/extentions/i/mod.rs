use terminus_global::*;
use std::rc::Rc;
use std::any::Any;
use crate::extentions::HasCsr;

mod insns;
pub mod csrs;

use csrs::ICsrs;

pub struct ExtensionI {
    csrs: Rc<ICsrs>,
}

impl ExtensionI {
    pub fn new(xlen: XLen) -> ExtensionI {
        ExtensionI {
            csrs: Rc::new(ICsrs::new(xlen))
        }
    }
}

impl HasCsr for ExtensionI {
    fn csrs(&self) -> Option<Rc<dyn Any>> {
        Some(self.csrs.clone() as Rc<dyn Any>)
    }
    fn csr_write(&self, addr: RegT, value: RegT) -> Option<()> {
        self.csrs.write(addr, value)
    }
    fn csr_read(&self, addr: RegT) -> Option<RegT> {
        self.csrs.read(addr)
    }
}

