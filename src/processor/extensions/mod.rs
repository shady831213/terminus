use crate::processor::ProcessorCfg;
use terminus_global::*;

pub mod a;
pub mod c;
pub mod d;
pub mod f;
pub mod i;
pub mod m;
pub mod v;

use a::*;
use c::*;
use d::*;
use f::*;
use i::*;
use m::*;
use v::*;
use std::rc::Rc;
use std::any::Any;

pub trait HasCsr {
    fn csrs(&self) -> Option<Rc<dyn Any>>;
    fn csr_write(&self, addr: RegT, value: RegT) -> Option<()>;
    fn csr_read(&self, addr: RegT) -> Option<RegT>;
}

trait NoCsr {
    fn csrs(&self) -> Option<Rc<dyn Any>> {
        None
    }
    fn csr_write(&self, _: RegT, _: RegT) -> Option<()> {
        None
    }
    fn csr_read(&self, _: RegT) -> Option<RegT> {
        None
    }
}


pub enum Extension {
    A(ExtensionA),
    C(ExtensionC),
    D(ExtensionD),
    F(ExtensionF),
    I(ExtensionI),
    M(ExtensionM),
    V(ExtensionV),
}

impl Extension {
    pub fn new(cfg: &ProcessorCfg, id: char) -> Result<Extension, String> {
        match id {
            'a' => Ok(Extension::A(ExtensionA {})),
            'c' => Ok(Extension::C(ExtensionC {})),
            'd' => Ok(Extension::D(ExtensionD {})),
            'f' => Ok(Extension::F(ExtensionF {})),
            'i' => Ok(Extension::I(ExtensionI::new(cfg))),
            'm' => Ok(Extension::M(ExtensionM::new(cfg))),
            'v' => Ok(Extension::V(ExtensionV {})),
            _ => Err(format!("unsupported extension \'{}\', supported extension is a, c, d, f, i, m or v!", id))
        }
    }
}

impl HasCsr for Extension {
    fn csrs(&self) -> Option<Rc<dyn Any>> {
        match self {
            Extension::A(a) => a.csrs(),
            Extension::C(c) => c.csrs(),
            Extension::D(d) => d.csrs(),
            Extension::F(f) => f.csrs(),
            Extension::I(i) => i.csrs(),
            Extension::M(m) => m.csrs(),
            Extension::V(v) => v.csrs(),
        }
    }
    fn csr_write(&self, addr: RegT, value: RegT) -> Option<()> {
        match self {
            Extension::A(a) => a.csr_write(addr, value),
            Extension::C(c) => c.csr_write(addr, value),
            Extension::D(d) => d.csr_write(addr, value),
            Extension::F(f) => f.csr_write(addr, value),
            Extension::I(i) => i.csr_write(addr, value),
            Extension::M(m) => m.csr_write(addr, value),
            Extension::V(v) => v.csr_write(addr, value),
        }
    }
    fn csr_read(&self, addr: RegT) -> Option<RegT> {
        match self {
            Extension::A(a) => a.csr_read(addr),
            Extension::C(c) => c.csr_read(addr),
            Extension::D(d) => d.csr_read(addr),
            Extension::F(f) => f.csr_read(addr),
            Extension::I(i) => i.csr_read(addr),
            Extension::M(m) => m.csr_read(addr),
            Extension::V(v) => v.csr_read(addr),
        }
    }
}