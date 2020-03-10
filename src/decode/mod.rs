#![allow(dead_code)]
#![allow(unused_imports)]

mod simple_insn_map;
mod tree_insn_map;

use std::sync::Arc;

use super::*;
use super::insn::*;
use super::execption::*;

use simple_insn_map::*;
use tree_insn_map::*;

pub trait Decoder {
    fn code(&self) -> u32;
    fn mask(&self) -> u32;
    fn matched(&self, ir: u32) -> bool;
    fn decode(&self, ir: u32) -> Instruction;
}

pub trait InsnMap {
    fn registery<T: 'static + Decoder>(&mut self, decoder: T);
    fn decode(&self, ir: u32) -> Result<Instruction, Exception>;
}

pub type GlobalInsnMap = TreeInsnMap;

impl GlobalInsnMap {
    pub fn get() -> Arc<GlobalInsnMap> {
        static mut MAP: Option<Arc<GlobalInsnMap>> = None;
        unsafe {
            MAP.get_or_insert_with(|| {
                Arc::new({
                    let mut map = GlobalInsnMap::new();
                    for r in REGISTERY_INSN {
                        r(&mut map)
                    }
                    map
                })
            }).clone()
        }
    }
}

#[distributed_slice]
pub static REGISTERY_INSN: [fn(&mut GlobalInsnMap)] = [..];