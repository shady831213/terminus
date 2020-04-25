use std::sync::{Arc, Mutex};
use terminus_spaceport::space::Space;
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
    #[cfg_attr(feature = "no-inline", inline(never))]
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
    #[cfg_attr(feature = "no-inline", inline(never))]
    pub fn lock_holder(&self, addr: u64, len: u64) -> Option<usize> {
        let lock_table = self.lock_table.lock().unwrap();
        if let Some(e) = lock_table.iter().find_map(|entry| { entry.lock_holder(addr, len) }) {
            Some(e)
        } else {
            None
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
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
    #[cfg_attr(feature = "no-inline", inline(never))]
    pub fn release(&self, who: usize) {
        let mut lock_table = self.lock_table.lock().unwrap();
        lock_table.retain(|e| { e.holder != who })
    }

    pub fn amo_u32<F: Fn(u32) -> u32>(&self, addr: u64, f: F) -> Result<u32, u64> {
        let read = self.read_u32(addr)?;
        let write = f(read);
        self.write_u32(addr, write)?;
        Ok(read)
    }
    pub fn amo_u64<F: Fn(u64) -> u64>(&self, addr: u64, f: F) -> Result<u64, u64> {
        let read = self.read_u64(addr)?;
        let write = f(read);
        self.write_u64(addr, write)?;
        Ok(read)
    }
}

impl Deref for Bus {
    type Target = Arc<Space>;
    fn deref(&self) -> &Self::Target {
        &self.space
    }
}