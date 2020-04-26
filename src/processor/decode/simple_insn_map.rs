use std::collections::HashMap;
use super::{InsnMap, Decoder};
use terminus_global::*;
use crate::processor::trap::Exception;
use crate::processor::insn::Instruction;


pub struct SimpleInsnMap(HashMap<InsnT, Box<dyn Decoder>>);

impl SimpleInsnMap {
    pub fn new() -> SimpleInsnMap {
        SimpleInsnMap(HashMap::new())
    }
}

impl InsnMap for SimpleInsnMap {
    fn registery<T: 'static + Decoder>(&mut self, decoder: T) {
        self.0.insert(decoder.code(), Box::new(decoder));
    }

    fn decode(&self, ir: InsnT) -> Result<&Instruction, Exception> {
        let decoder = self.0.values().find(|d| { d.matched(ir) });
        if let Some(d) = decoder {
            Ok(d.decode())
        } else {
            Err(Exception::IllegalInsn(ir))
        }
    }
}