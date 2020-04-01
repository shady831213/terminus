use terminus_spaceport::memory::region::{BytesAccess, U8Access, U16Access, U32Access, U64Access, IOAccess};
use terminus_spaceport::{derive_io, EXIT_CTRL};
use terminus_spaceport::devices::TERM;
use terminus_spaceport::memory::region;
use std::sync::Mutex;
use std::io::{Write, ErrorKind, Read};
use terminus_macros::*;
use std::borrow::{BorrowMut, Borrow};

// test refer to top_tests/htif_test
struct HTIFDesp {
    tohost: u64,
    fromhost: u64,
}

impl HTIFDesp {
    fn tohost_cmd(&self) -> u64 {
        ((self.tohost) >> 48) & 0xff
    }
    fn tohost_device(&self) -> u64 {
        (self.tohost) >> 56
    }
    // fn fromhost_cmd(&self) -> u64 {
    //     ((self.fromhost) >> 48) & 0xff
    // }
    // fn fromhost_device(&self) -> u64 {
    //     (self.fromhost) >> 56
    // }
}

#[derive_io(Bytes, U32, U64)]
pub struct HTIF(Mutex<HTIFDesp>);

impl HTIF {
    pub fn new() -> HTIF {
        HTIF(Mutex::new(HTIFDesp { tohost: 0, fromhost: 0 }))
    }

    fn handle_cmd(desp: &mut HTIFDesp) -> region::Result<()> {
        if desp.tohost == 1 {
            EXIT_CTRL.exit("htif shutdown!").unwrap();
            Ok(())
        } else if desp.tohost_device() == 1 && desp.tohost_cmd() == 1 {
            let mut data = [0u8; 1];
            data[0] = desp.tohost as u8;
            let stdout = TERM.stdout();
            stdout.lock().write(&data).unwrap();
            stdout.lock().flush().unwrap();
            desp.tohost = 0;
            Ok(())
        } else if desp.tohost_device() == 1 && desp.tohost_cmd() == 0 {
            desp.tohost = 0;
            Ok(())
        } else {
            Err(region::Error::AccessErr(desp.tohost, format!("HTIF:unsupportd tohost={:#x}", desp.tohost)))
        }
    }

    fn fromhost_poll(&self) {
        let mut desp = self.0.lock().unwrap();
        if desp.borrow().fromhost == 0 {
            let mut data = [0u8; 1];
            match TERM.stdin().lock().read_exact(&mut data) {
                Ok(_) => desp.borrow_mut().fromhost.set_bit_range(7, 0, data[0]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => panic!("{:?}", e)
            }
        }
    }
}

impl BytesAccess for HTIF {
    fn write(&self, _: u64, _: &[u8]) -> region::Result<()> {Ok(())}
}

impl U32Access for HTIF {
    fn write(&self, addr: u64, data: u32) -> region::Result<()> {
        match addr {
            0x0 => Ok(self.0.lock().unwrap().tohost.set_bit_range(31, 0, data)),
            0x4 => {
                let mut desp = self.0.lock().unwrap();
                desp.borrow_mut().tohost.set_bit_range(63, 32, data);
                HTIF::handle_cmd(desp.borrow_mut())
            }
            0x8 => Ok(self.0.lock().unwrap().fromhost.set_bit_range(31, 0, data)),
            0xc => Ok(self.0.lock().unwrap().fromhost.set_bit_range(63, 32, data)),
            _ => Err(region::Error::AccessErr(addr, "invalid HTIF addr".to_string()))
        }
    }

    fn read(&self, addr: u64) -> region::Result<u32> {
        self.fromhost_poll();
        match addr {
            0x0 => Ok(self.0.lock().unwrap().tohost.bit_range(31, 0)),
            0x4 => Ok(self.0.lock().unwrap().tohost.bit_range(63, 32)),
            0x8 => Ok(self.0.lock().unwrap().fromhost.bit_range(31, 0)),
            0xc => Ok(self.0.lock().unwrap().fromhost.bit_range(63, 32)),
            _ => Err(region::Error::AccessErr(addr, "invalid HTIF addr".to_string()))
        }
    }
}


impl U64Access for HTIF {
    fn write(&self, addr: u64, data: u64) -> region::Result<()> {
        match addr {
            0x0 => {
                let mut desp = self.0.lock().unwrap();
                desp.borrow_mut().tohost = data;
                HTIF::handle_cmd(desp.borrow_mut())
            }
            0x8 => Ok(self.0.lock().unwrap().fromhost = data),
            _ => Err(region::Error::AccessErr(addr, "invalid HTIF addr".to_string()))
        }
    }

    fn read(&self, addr: u64) -> region::Result<u64> {
        self.fromhost_poll();
        match addr {
            0x0 => Ok(self.0.lock().unwrap().tohost),
            0x8 => Ok(self.0.lock().unwrap().fromhost),
            _ => Err(region::Error::AccessErr(addr, "invalid HTIF addr".to_string()))
        }
    }
}



