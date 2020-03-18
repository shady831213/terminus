use terminus_spaceport::memory::{BytesAccess, U8Access, U16Access, U32Access, U64Access, IOAccess};
use terminus_spaceport::ts_io;
use std::sync::Mutex;
use terminus_macros::*;

struct HTIFDesp {
    tohost: u64,
    fromhost: u64,
}

impl HTIFDesp {
    fn cmd(&self) -> u32 {
        ((self.tohost as u32) >> 48) & 0xff
    }
    fn device(&self) -> u32 {
        (self.tohost as u32) >> 56
    }
}

#[ts_io(Bytes, U32)]
pub struct HTIF(Mutex<HTIFDesp>);

impl HTIF {
    pub fn new() -> HTIF {
        HTIF(Mutex::new(HTIFDesp { tohost: 0, fromhost: 0 }))
    }
}

impl BytesAccess for HTIF {
    fn write(&self, _: u64, _: &[u8]) {}

    fn read(&self, _: u64, _: &mut [u8]) {
        panic!("HTIF BytesAccess::read not implement!")
    }
}

impl U32Access for HTIF {
    fn write(&self, addr: u64, data: u32) {
        match addr {
            0x0 => self.0.lock().unwrap().tohost.set_bit_range(31, 0, data),
            0x4 => self.0.lock().unwrap().tohost.set_bit_range(63, 32, data),
            0x8 => self.0.lock().unwrap().fromhost.set_bit_range(31, 0, data),
            0xc => self.0.lock().unwrap().fromhost.set_bit_range(63, 32, data),
            _ => {}
        }
    }

    fn read(&self, addr: u64) -> u32 {
        match addr {
            0x0 => self.0.lock().unwrap().tohost.bit_range(31, 0),
            0x4 => self.0.lock().unwrap().tohost.bit_range(63, 32),
            0x8 => self.0.lock().unwrap().fromhost.bit_range(31, 0),
            0xc => self.0.lock().unwrap().fromhost.bit_range(63, 32),
            _ => 0
        }
    }
}


