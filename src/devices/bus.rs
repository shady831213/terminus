use terminus_spaceport::memory::prelude::*;
use std::sync::{Arc, Mutex};
use terminus_spaceport::space::Space;
use terminus_spaceport::memory::region;
use std::ops::Deref;


#[derive(Debug)]
struct LockEntry {
    addr: u64,
    len: u64,
    holder: usize,
}

impl LockEntry {
    fn lock_holder(&self, addr: u64, len: u64) -> Option<usize> {
        if addr >= self.addr && addr < self.addr + self.len || addr + len - 1 >= self.addr && addr + len - 1 < self.addr + self.len ||
            self.addr >= addr && self.addr < addr + len || self.addr + self.len - 1 >= addr && self.addr + self.len - 1 < addr + len {
            Some(self.holder)
        } else {
            None
        }
    }
}

#[derive_io(U8, U16, U32, U64)]
pub struct Bus {
    space: Arc<Space>,
    lock_table: Mutex<Vec<LockEntry>>,
}

impl Bus {
    pub fn new(space: &Arc<Space>) -> Bus {
        Bus {
            space: space.clone(),
            lock_table: Mutex::new(vec![]),
        }
    }
    pub fn acquire(&self, addr: u64, len: u64, who: usize) -> bool {
        let mut lock_table = self.lock_table.lock().unwrap();
        if lock_table.iter().find(|entry| {
            if let Some(lock_owner) = entry.lock_holder(addr, len) {
                if who == lock_owner {
                    panic!(format!("master {} try to lock {:#x} - {:#x} twice!", who, addr, addr + len))
                }
                true
            } else {
                false
            }
        }).is_some() {
            false
        } else {
            lock_table.push(LockEntry {
                addr,
                len,
                holder: who,
            });
            true
        }
    }

    pub fn lock_holder(&self, addr: u64, len: u64) -> Option<usize> {
        let lock_table = self.lock_table.lock().unwrap();
        if let Some(e) = lock_table.iter().find_map(|entry| { entry.lock_holder(addr, len) }) {
            Some(e)
        } else {
            None
        }
    }

    pub fn invalid_lock(&self, addr: u64, len: u64, who: usize) {
        let mut lock_table = self.lock_table.lock().unwrap();
        if let Some((i, _)) = lock_table.iter().enumerate().find(|(_, entry)| {
            if let Some(lock_owner) = entry.lock_holder(addr, len) {
                if who == lock_owner {
                    true
                } else {
                    panic!(format!("master {} try to release {:#x} - {:#x} but haven't owned the lock! lock_table:{:?}", who, addr, addr + len, lock_table))
                }
            } else {
                false
            }
        }) {
            lock_table.remove(i);
        } else {
            panic!(format!("master {} try to release {:#x} - {:#x} but haven't owned the lock! lock_table:{:?}", who, addr, addr + len, lock_table))
        }
    }

    pub fn release(&self, who: usize) {
        let mut lock_table = self.lock_table.lock().unwrap();
        lock_table.retain(|e|{e.holder != who})
    }

    pub fn amo_u32<F: Fn(u32) -> u32>(&self, addr: u64, f: F) -> region::Result<u32> {
        let read = U32Access::read(self.space.deref(), addr)?;
        let write = f(read);
        U32Access::write(self.space.deref(), addr, write)?;
        Ok(read)
    }
    pub fn amo_u64<F: Fn(u64) -> u64>(&self, addr: u64, f: F) -> region::Result<u64> {
        let read = U64Access::read(self.space.deref(), addr)?;
        let write = f(read);
        U64Access::write(self.space.deref(), addr, write)?;
        Ok(read)
    }
}

impl U8Access for Bus {
    fn write(&self, addr: u64, data: u8) -> region::Result<()> {
        U8Access::write(self.space.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u8> {
        U8Access::read(self.space.deref(), addr)
    }
}

impl U16Access for Bus {
    fn write(&self, addr: u64, data: u16) -> region::Result<()> {
        U16Access::write(self.space.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u16> {
        U16Access::read(self.space.deref(), addr)
    }
}

impl U32Access for Bus {
    fn write(&self, addr: u64, data: u32) -> region::Result<()> {
        U32Access::write(self.space.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u32> {
        U32Access::read(self.space.deref(), addr)
    }
}

impl U64Access for Bus {
    fn write(&self, addr: u64, data: u64) -> region::Result<()> {
        U64Access::write(self.space.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u64> {
        U64Access::read(self.space.deref(), addr)
    }
}
