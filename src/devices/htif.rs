use terminus_spaceport::memory::prelude::*;
use terminus_spaceport::EXIT_CTRL;
use terminus_spaceport::devices::TERM;
use std::io::{Write, ErrorKind, Read};
use terminus_vault::*;
use std::borrow::{BorrowMut, Borrow};
use std::cell::RefCell;

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
    desc: RefCell<HTIFDesp>,
    tohost_off: u64,
    fromhost_off: Option<u64>,
    input_en: bool,
}

impl HTIF {
    pub fn new(tohost_off: u64, fromhost_off: Option<u64>, input_en: bool) -> HTIF {
        HTIF {
            desc: RefCell::new(HTIFDesp { tohost: 0, fromhost: 0 }),
            tohost_off,
            fromhost_off,
            input_en,
        }
    }

    fn handle_cmd(&self, desp: &mut HTIFDesp) {
        if desp.tohost_device() == 0 && desp.tohost_cmd() == 0 {
            if desp.tohost & 0x1 == 1 {
                EXIT_CTRL.exit("htif shutdown!").unwrap();
            }
        } else if desp.tohost_device() == 1 && desp.tohost_cmd() == 1 {
            let mut data = [0u8; 1];
            data[0] = desp.tohost as u8;
            let stdout = TERM.stdout();
            let mut handle = stdout.lock();
            handle.write(&data).unwrap();
            handle.flush().unwrap();
            desp.tohost = 0;
        } else if desp.tohost_device() == 1 && desp.tohost_cmd() == 0 {
            desp.tohost = 0;
        } else {
            panic!(format!("unsupported cmd:{:#x}", *desp.tohost.borrow()))
        }
    }

    fn fromhost_poll(&self, desp: &mut HTIFDesp) {
        if desp.fromhost == 0 && self.input_en {
            let mut data = [0u8; 1];
            match TERM.stdin().lock().read_exact(&mut data) {
                Ok(_) => {
                    desp.fromhost.set_bit_range(8, 8, 1);
                    desp.fromhost.set_bit_range(7, 0, data[0]);
                    desp.fromhost.set_bit_range(63, 48, 0x0100);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => panic!("{:?}", e)
            }
        }
    }
}

impl BytesAccess for HTIF {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        if data.len() == 4 {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(data);
            U32Access::write(self, addr, u32::from_le_bytes(bytes))
        } else if data.len() == 8 {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(data);
            U64Access::write(self, addr, u64::from_le_bytes(bytes))
        }
        Ok(0)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        if data.len() == 4 {
            data.copy_from_slice(&U32Access::read(self, addr).to_le_bytes())
        } else if data.len() == 8 {
            data.copy_from_slice(&U64Access::read(self, addr).to_le_bytes())
        }
        Ok(0)
    }
}

impl U32Access for HTIF {
    fn write(&self, addr: &u64, data: u32) {
        let mut desp = self.desc.borrow_mut();
        if *addr == self.tohost_off {
            desp.borrow_mut().tohost.set_bit_range(31, 0, data);
            self.handle_cmd(desp.borrow_mut())
        } else if *addr == self.tohost_off + 4 {
            let mut desp = self.desc.borrow_mut();
            desp.borrow_mut().tohost.set_bit_range(63, 32, data);
        } else if let Some(fromhost) = self.fromhost_off {
            if *addr == fromhost {
                desp.fromhost.set_bit_range(31, 0, data)
            } else if *addr == fromhost + 4 {
                desp.fromhost.set_bit_range(63, 32, data)
            }
        }
    }

    fn read(&self, addr: &u64) -> u32 {
        let mut desp = self.desc.borrow_mut();
        if *addr == self.tohost_off {
            desp.tohost as u32
        } else if *addr == self.tohost_off + 4 {
            (desp.tohost >> 32) as u32
        } else if let Some(fromhost) = self.fromhost_off {
            if *addr == fromhost {
                self.fromhost_poll(desp.borrow_mut());
                desp.fromhost as u32
            } else if *addr == fromhost + 4 {
                self.fromhost_poll(desp.borrow_mut());
                (desp.fromhost >> 32) as u32
            } else {
                panic!("invalid HTIF addr")
            }
        } else {
            panic!("invalid HTIF addr")
        }
    }
}


impl U64Access for HTIF {
    fn write(&self, addr: &u64, data: u64) {
        let mut desp = self.desc.borrow_mut();
        if *addr == self.tohost_off {
            desp.borrow_mut().tohost = data;
            self.handle_cmd(desp.borrow_mut())
        } else if let Some(fromhost) = self.fromhost_off {
            if *addr == fromhost {
                desp.fromhost = data
            } else {
                panic!("invalid HTIF addr")
            }
        } else {
            panic!("invalid HTIF addr")
        }
    }

    fn read(&self, addr: &u64) -> u64 {
        let mut desp = self.desc.borrow_mut();
        let data = if *addr == self.tohost_off {
            desp.tohost
        } else if let Some(fromhost) = self.fromhost_off {
            if *addr == fromhost {
                self.fromhost_poll(desp.borrow_mut());
                desp.fromhost
            } else {
                panic!("invalid HTIF addr");
            }
        } else {
            panic!("invalid HTIF addr");
        };
        data
    }
}



