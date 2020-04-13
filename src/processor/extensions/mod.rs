use crate::processor::ProcessorState;
use terminus_global::*;

pub mod a;
pub mod c;
pub mod d;
pub mod f;
pub mod i;
pub mod m;
pub mod s;
pub mod u;
pub mod v;

use a::*;
use c::*;
use d::*;
use f::*;
use i::*;
use m::*;
use s::*;
use u::*;
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
    A(Rc<ExtensionA>),
    C(Rc<ExtensionC>),
    D(Rc<ExtensionD>),
    F(Rc<ExtensionF>),
    I(Rc<ExtensionI>),
    M(Rc<ExtensionM>),
    S(Rc<ExtensionS>),
    U(Rc<ExtensionU>),
    V(Rc<ExtensionV>),
}

impl Extension {
    pub fn new(state: &ProcessorState, id: char) -> Result<Extension, String> {
        match id {
            'a' => Ok(Extension::A(Rc::new(ExtensionA {}))),
            'c' => Ok(Extension::C(Rc::new(ExtensionC::new(state.config())))),
            'd' => Ok(Extension::D(Rc::new(ExtensionD {}))),
            'f' => Ok(Extension::F(Rc::new(ExtensionF::new(state)))),
            'i' => Ok(Extension::I(Rc::new(ExtensionI::new(state.config())))),
            'm' => Ok(Extension::M(Rc::new(ExtensionM::new(state.config())))),
            's' => Ok(Extension::S(Rc::new(ExtensionS::new(state)))),
            'u' => Ok(Extension::U(Rc::new(ExtensionU {}))),
            'v' => Ok(Extension::V(Rc::new(ExtensionV {}))),
            _ => Err(format!("unsupported extension \'{}\', supported extension is a, c, d, f, i, m, s, u or v!", id))
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
            Extension::S(s) => s.csrs(),
            Extension::U(u) => u.csrs(),
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
            Extension::S(s) => s.csr_write(addr, value),
            Extension::U(u) => u.csr_write(addr, value),
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
            Extension::S(s) => s.csr_read(addr),
            Extension::U(u) => u.csr_read(addr),
            Extension::V(v) => v.csr_read(addr),
        }
    }
}