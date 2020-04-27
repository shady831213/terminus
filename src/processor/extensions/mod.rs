use crate::processor::{ProcessorState, Processor};
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

trait HasCsr {
    fn csr_write(&self, state: &ProcessorState, addr: InsnT, value: RegT) -> Option<()>;
    fn csr_read(&self, state: &ProcessorState, addr: InsnT) -> Option<RegT>;
}


trait NoCsr {
    fn csr_write(&self, _: &ProcessorState, _: InsnT, _: RegT) -> Option<()> {
        None
    }
    fn csr_read(&self, _: &ProcessorState, _: InsnT) -> Option<RegT> {
        None
    }
}

trait HasStepCb {
    fn step_cb(&self, p: &Processor);
}

trait NoStepCb {
    fn step_cb(&self, _: &Processor) {}
}


pub enum Extension {
    A(ExtensionA),
    C(ExtensionC),
    D(ExtensionD),
    F(ExtensionF),
    I(ExtensionI),
    M(ExtensionM),
    S(ExtensionS),
    U(ExtensionU),
    V(ExtensionV),
    None
}

impl Extension {
    pub fn new(state: &ProcessorState, id: char) -> Result<Extension, String> {
        match id {
            'a' => Ok(Extension::A(ExtensionA::new(state))),
            'c' => Ok(Extension::C(ExtensionC::new(state))),
            'd' => Ok(Extension::D(ExtensionD::new(state))),
            'f' => Ok(Extension::F(ExtensionF::new(state))),
            'i' => Ok(Extension::I(ExtensionI::new(state))),
            'm' => Ok(Extension::M(ExtensionM::new(state))),
            's' => Ok(Extension::S(ExtensionS::new(state))),
            'u' => Ok(Extension::U(ExtensionU::new(state))),
            // 'v' => Ok(Extension::V(Rc::new(ExtensionV {}))),
            _ => Err(format!("unsupported extension \'{}\', supported extension is a, c, d, f, i, m, s, u!", id))
        }
    }
    // pub fn name(&self) -> Option<char> {
    //     match self {
    //         Extension::A(a) => Some('a'),
    //         Extension::C(c) => Some('c'),
    //         Extension::D(d) => Some('d'),
    //         Extension::F(f) => Some('f'),
    //         Extension::I(i) => Some('i'),
    //         Extension::M(m) => Some('m'),
    //         Extension::S(s) => Some('s'),
    //         Extension::U(u) => Some('u'),
    //         Extension::V(v) => Some('v'),
    //         _ => None
    //     }
    // }
    pub fn csr_write(&self, state: &ProcessorState, addr: InsnT, value: RegT) -> Option<()> {
        match self {
            Extension::A(a) => a.csr_write(state, addr, value),
            Extension::C(c) => c.csr_write(state, addr, value),
            Extension::D(d) => d.csr_write(state, addr, value),
            Extension::F(f) => f.csr_write(state, addr, value),
            Extension::I(i) => i.csr_write(state, addr, value),
            Extension::M(m) => m.csr_write(state, addr, value),
            Extension::S(s) => s.csr_write(state, addr, value),
            Extension::U(u) => u.csr_write(state, addr, value),
            Extension::V(v) => v.csr_write(state, addr, value),
            _  => None
        }
    }
    pub fn csr_read(&self, state: &ProcessorState, addr: InsnT) -> Option<RegT> {
        match self {
            Extension::A(a) => a.csr_read(state, addr),
            Extension::C(c) => c.csr_read(state, addr),
            Extension::D(d) => d.csr_read(state, addr),
            Extension::F(f) => f.csr_read(state, addr),
            Extension::I(i) => i.csr_read(state, addr),
            Extension::M(m) => m.csr_read(state, addr),
            Extension::S(s) => s.csr_read(state, addr),
            Extension::U(u) => u.csr_read(state, addr),
            Extension::V(v) => v.csr_read(state, addr),
            _  => None
        }
    }

    pub fn step_cb(&self, p: &Processor) {
        match self {
            Extension::A(a) => a.step_cb(p),
            Extension::C(c) => c.step_cb(p),
            Extension::D(d) => d.step_cb(p),
            Extension::F(f) => f.step_cb(p),
            Extension::I(i) => i.step_cb(p),
            Extension::M(m) => m.step_cb(p),
            Extension::S(s) => s.step_cb(p),
            Extension::U(u) => u.step_cb(p),
            Extension::V(v) => v.step_cb(p),
            _ => {}
        }
    }
}