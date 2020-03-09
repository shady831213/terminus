use std::collections::HashMap;
use super::{InsnMap, Instruction, Decoder};

pub struct SimpleInsnMap(HashMap<u32, Box<dyn Decoder>>);

impl SimpleInsnMap {
    pub fn new() -> SimpleInsnMap {
        SimpleInsnMap(HashMap::new())
    }
}

impl InsnMap for SimpleInsnMap {
    fn registery<T: 'static + Decoder>(&mut self, decoder: T) {
        self.0.insert(decoder.code(), Box::new(decoder));
    }

    fn decode(&self, ir: u32) -> Result<Instruction, String> {
        let decoder = self.0.values().find(|d| { d.matched(ir) });
        if let Some(d) = decoder {
            Ok(d.decode(ir))
        } else {
            Err("invalid instruction!".to_string())
        }
    }
}