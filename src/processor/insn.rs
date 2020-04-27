use std::ops::Deref;
use crate::processor::Processor;
use terminus_global::*;
use crate::processor::trap::Exception;

pub trait Format {
    fn ir(&self, code: InsnT) -> InsnT { code }
    fn rs1(&self, _: InsnT) -> InsnT {
        0
    }
    fn rs2(&self, _: InsnT) -> InsnT {
        0
    }
    fn rd(&self, _: InsnT) -> InsnT {
        0
    }
    fn imm(&self, _: InsnT) -> InsnT {
        0
    }
    fn op(&self, _: InsnT) -> InsnT {
        0
    }
    fn imm_len(&self) -> usize {
        0
    }
}

pub trait Execution {
    fn execute(&self, p: &mut Processor) -> Result<(), Exception>;
}


pub trait InstructionImp: Format + Execution + Send + Sync {}

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



