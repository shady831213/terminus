#![allow(dead_code)]
#![allow(unused_imports)]
mod simple_insn_map;
mod tree_insn_map;

use std::sync::Arc;

use terminus_global::InsnT;
use crate::linkme::*;

use simple_insn_map::*;
use tree_insn_map::*;
use crate::processor::insn::Instruction;
use crate::processor::trap::Exception;

pub trait Decoder:Send+Sync {
    fn code(&self) -> InsnT;
    fn mask(&self) -> InsnT;
    fn matched(&self, ir: InsnT) -> bool;
    fn decode(&self, ir: InsnT) -> Instruction;
    fn name(&self) -> String;
}

pub trait InsnMap {
    fn registery<T: 'static + Decoder>(&mut self, decoder: T);
    fn decode(&self, ir: InsnT) -> Result<Instruction, Exception>;
    fn lock(&mut self) {}
}

pub type GlobalInsnMap = TreeInsnMap;

lazy_static! {
    pub static ref GDECODER:GlobalInsnMap = {
        let mut map = GlobalInsnMap::new();
        for r in REGISTERY_INSN {
            r(&mut map)
        }
        map.lock();
        map
    };
}

#[distributed_slice]
pub static REGISTERY_INSN: [fn(&mut GlobalInsnMap)] = [..];