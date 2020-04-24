use terminus_spaceport::memory::prelude::*;
use terminus_spaceport::EXIT_CTRL;
use terminus_spaceport::devices::TERM;
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
}

#[derive_io(Bytes, U32, U64)]
pub struct HTIF {
    desc: Mutex<HTIFDesp>,
    tohost_off: u64,
    fromhost_off: Option<u64>,
}

impl HTIF {
    pub fn new(tohost_off: u64, fromhost_off: Option<u64>) -> HTIF {
        HTIF {
            desc: Mutex::new(HTIFDesp { tohost: 0, fromhost: 0 }),
            tohost_off,
            fromhost_off,
        }
    }

    fn handle_cmd(desp: &mut HTIFDesp) {
        if desp.tohost & 0x1 == 1 && desp.tohost_device() == 0 && desp.tohost_cmd() == 0 {
            EXIT_CTRL.exit("htif shutdown!").unwrap();
        } else if desp.tohost_device() == 1 && desp.tohost_cmd() == 1 {
            let mut data = [0u8; 1];
            data[0] = desp.tohost as u8;
            let stdout = TERM.stdout();
            let mut handle = stdout.lock();
            handle.write(&data).unwrap();
            handle.flush().unwrap();
            desp.tohost = 0;
            desp.fromhost = desp.tohost_device() << 56 | desp.tohost_cmd() << 48;
        } else if desp.tohost_device() == 1 && desp.tohost_cmd() == 0 {
            desp.tohost = 0;
        } else {
            panic!(format!("HTIF:unsupportd tohost={:#x}", desp.tohost))
        }
    }

    fn fromhost_poll(&self) {
        let mut desp = self.desc.lock().unwrap();
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
    fn write(&self, _: u64, _: &[u8]) {  }
}

impl U32Access for HTIF {
    fn write(&self, addr: u64, data: u32) {
        if addr == self.tohost_off {
            let mut desp = self.desc.lock().unwrap();
            desp.borrow_mut().tohost.set_bit_range(31, 0, data);
            if desp.borrow().tohost & 0x1 == 1 && desp.tohost_device() == 0 && desp.tohost_cmd() == 0 {
                EXIT_CTRL.exit("htif shutdown!").unwrap();
            }
        } else if addr == self.tohost_off + 4 {
            let mut desp = self.desc.lock().unwrap();
            desp.borrow_mut().tohost.set_bit_range(63, 32, data);
            HTIF::handle_cmd(desp.borrow_mut())
        } else if let Some(fromhost) = self.fromhost_off {
            if addr == fromhost {
                self.desc.lock().unwrap().fromhost.set_bit_range(31, 0, data)
            } else if addr == fromhost + 4 {
                self.desc.lock().unwrap().fromhost.set_bit_range(63, 32, data)
            } else {
                panic!("invalid HTIF addr")
            }
        } else {
            panic!("invalid HTIF addr")
        }
    }

    fn read(&self, addr: u64) -> u32 {
        if addr == self.tohost_off {
            self.desc.lock().unwrap().tohost as u32
        } else if addr == self.tohost_off + 4 {
            (self.desc.lock().unwrap().tohost >> 32) as u32
        } else if let Some(fromhost) = self.fromhost_off {
            if addr == fromhost {
                self.fromhost_poll();
                self.desc.lock().unwrap().fromhost as u32
            } else if addr == fromhost + 4 {
                self.fromhost_poll();
                (self.desc.lock().unwrap().fromhost >> 32) as u32
            } else {
                panic!("invalid HTIF addr")
            }
        } else {
            panic!("invalid HTIF addr")
        }
    }
}


impl U64Access for HTIF {
    fn write(&self, addr: u64, data: u64) {
        if addr == self.tohost_off {
            let mut desp = self.desc.lock().unwrap();
            desp.borrow_mut().tohost = data;
            HTIF::handle_cmd(desp.borrow_mut())
        } else if let Some(fromhost) = self.fromhost_off {
            if addr == fromhost {
                self.desc.lock().unwrap().fromhost = data
            } else {
                panic!("invalid HTIF addr")
            }
        } else {
            panic!("invalid HTIF addr")
        }
    }

    fn read(&self, addr: u64) -> u64 {
        if addr == self.tohost_off {
            self.desc.lock().unwrap().tohost
        } else if let Some(fromhost) = self.fromhost_off {
            if addr == fromhost {
                self.fromhost_poll();
                self.desc.lock().unwrap().fromhost
            } else {
                panic!("invalid HTIF addr")
            }
        } else {
            panic!("invalid HTIF addr")
        }
    }
}



