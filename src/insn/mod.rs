use std::ops::Deref;
use super::processor::Processor;
use terminus_global::*;
use crate::Exception;

pub trait Format {
    fn ir(&self) -> InsnT;
    fn rs1(&self) -> InsnT {
        0
    }
    fn rs2(&self) -> InsnT {
        0
    }
    fn rs3(&self) -> InsnT {
        0
    }
    fn rd(&self) -> InsnT {
        0
    }
    fn imm(&self) -> InsnT {
        0
    }
    fn op(&self) -> InsnT {
        0
    }
}

pub trait Execution {
    fn execute(&self, p: &Processor) -> Result<(), Exception>;
}


pub trait InstructionImp: Format + Execution {}

pub struct Instruction(Box<dyn InstructionImp>);

impl Instruction {
    pub fn new<T: 'static + InstructionImp>(f: T) -> Instruction {
        Instruction(Box::new(f))
    }
}

impl Deref for Instruction {
    type Target = Box<dyn InstructionImp>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



