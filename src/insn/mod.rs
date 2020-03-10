use std::ops::Deref;
use super::processor::Processor;
pub trait Format {
    fn ir(&self) -> u32;
    fn rs1(&self) -> u32 {
        0
    }
    fn rs2(&self) -> u32 {
        0
    }
    fn rs3(&self) -> u32 {
        0
    }
    fn rd(&self) -> u32 {
        0
    }
    fn imm(&self) -> u32 {
        0
    }
    fn op(&self) -> u32 {
        0
    }
}

pub trait Execution {
    fn execute(&self, p: &mut Processor);
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



