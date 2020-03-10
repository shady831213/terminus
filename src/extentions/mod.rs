use crate::processor::Processor;
use crate::insn::Instruction;

mod a;
mod c;
mod d;
mod f;
mod i;
mod m;
mod v;

use a::*;
use c::*;
use d::*;
use f::*;
use i::*;
use m::*;
use v::*;

pub enum Extension {
    A(ExtensionA),
    C(ExtensionC),
    D(ExtensionD),
    F(ExtensionF),
    I(ExtensionF),
    M(ExtensionM),
    V(ExtensionV),
}