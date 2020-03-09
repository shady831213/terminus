use std::ops::Deref;
use std::sync::Arc;

mod insn_maps;

use insn_maps::*;



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
    fn execute(&self);
}

pub trait Decoder {
    fn code(&self) -> u32;
    fn mask(&self) -> u32;
    fn matched(&self, ir: u32) -> bool;
    fn decode(&self, ir: u32) -> Instruction;
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

pub trait InsnMap {
    fn registery<T: 'static + Decoder>(&mut self, decoder: T);
    //fixme:Error should be exception enum
    fn decode(&self, ir: u32) -> Result<Instruction, String>;
}


pub type GlobalInsnMap = SimpleInsnMap;

impl GlobalInsnMap {
    pub fn get() ->  Arc<GlobalInsnMap> {
        static mut Table: Option<Arc<GlobalInsnMap>> = None;
        unsafe {
            Table.get_or_insert_with(|| {
                Arc::new({let mut map = GlobalInsnMap::new();
                    for r in REGISTERY_INSN {
                        r(&mut map)
                    }
                map})
            }).clone()
        }
    }
}

use linkme::distributed_slice;
#[distributed_slice]
pub static REGISTERY_INSN: [fn(&mut GlobalInsnMap)] = [..];
