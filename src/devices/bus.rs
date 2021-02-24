use std::cell::{Ref, RefCell, RefMut};
use terminus_spaceport::space::Space;

#[derive(Debug)]
struct LockEntry {
    addr: u64,
    len: usize,
    holder: usize,
}

impl LockEntry {
    fn lock_holder(&self, addr: &u64, len: usize) -> Option<usize> {
        if *addr >= self.addr && *addr < self.addr + self.len as u64
            || *addr + len as u64 - 1 >= self.addr
                && *addr + len as u64 - 1 < self.addr + self.len as u64
            || self.addr >= *addr && self.addr < *addr + len as u64
            || self.addr + self.len as u64 - 1 >= *addr
                && self.addr + self.len as u64 - 1 < *addr + len as u64
        {
            Some(self.holder)
        } else {
            None
        }
    }
}

pub trait Bus {
    fn acquire(&self, _addr: &u64, _len: usize, _who: usize) -> bool {
        panic!("acquire is not supported!")
    }
    fn lock_holder(&self, _addr: &u64, _len: usize) -> Option<usize> {
        panic!("lock_holder is not supported!")
    }
    fn invalid_lock(&self, _addr: &u64, _len: usize, _who: usize){
        panic!("invalid_lock is not supported!")
    }
    fn release(&self, _who: usize) {
        panic!("release is not supported!")
    }
    fn write_u8(&self, addr: &u64, data: &u8) -> Result<(), u64>;
    fn read_u8(&self, addr: &u64, data: &mut u8) -> Result<(), u64>;
    fn write_u16(&self, addr: &u64, data: &u16) -> Result<(), u64>;
    fn read_u16(&self, addr: &u64, data: &mut u16) -> Result<(), u64>;
    fn write_u32(&self, addr: &u64, data: &u32) -> Result<(), u64>;
    fn read_u32(&self, addr: &u64, data: &mut u32) -> Result<(), u64>;
    fn write_u64(&self, addr: &u64, data: &u64) -> Result<(), u64>;
    fn read_u64(&self, addr: &u64, data: &mut u64) -> Result<(), u64>;
}

pub struct TerminusBus {
    space: RefCell<Space>,
    lock_table: RefCell<Vec<LockEntry>>,
}

impl TerminusBus {
    pub fn new() -> TerminusBus {
        TerminusBus {
            space: RefCell::new(Space::new()),
            lock_table: RefCell::new(vec![]),
        }
    }
    pub fn space(&self) -> Ref<'_, Space> {
        self.space.borrow()
    }

    pub fn space_mut(&self) -> RefMut<'_, Space> {
        self.space.borrow_mut()
    }
}

impl Bus for TerminusBus {
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn acquire(&self, addr: &u64, len: usize, who: usize) -> bool {
        let mut lock_table = self.lock_table.borrow_mut();
        if lock_table
            .iter()
            .find(|entry| {
                if let Some(lock_owner) = entry.lock_holder(addr, len) {
                    if who == lock_owner {
                        panic!(format!(
                            "master {} try to lock {:#x} - {:#x} twice!",
                            who,
                            *addr,
                            *addr + len as u64
                        ))
                    }
                    true
                } else {
                    false
                }
            })
            .is_some()
        {
            false
        } else {
            lock_table.push(LockEntry {
                addr: *addr,
                len,
                holder: who,
            });
            true
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn lock_holder(&self, addr: &u64, len: usize) -> Option<usize> {
        let lock_table = self.lock_table.borrow();
        if let Some(e) = lock_table
            .iter()
            .find_map(|entry| entry.lock_holder(addr, len))
        {
            Some(e)
        } else {
            None
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn invalid_lock(&self, addr: &u64, len: usize, who: usize) {
        let mut lock_table = self.lock_table.borrow_mut();
        if let Some((i, _)) = lock_table.iter().enumerate().find(|(_, entry)| {
            if let Some(lock_owner) = entry.lock_holder(addr, len) {
                if who == lock_owner {
                    true
                } else {
                    panic!(format!("master {} try to release {:#x} - {:#x} but haven't owned the lock! lock_table:{:?}", who, *addr, *addr + len as u64, lock_table))
                }
            } else {
                false
            }
        }) {
            lock_table.remove(i);
        } else {
            panic!(format!("master {} try to release {:#x} - {:#x} but haven't owned the lock! lock_table:{:?}", who, addr, *addr + len as u64, lock_table))
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn release(&self, who: usize) {
        let mut lock_table = self.lock_table.borrow_mut();
        lock_table.retain(|e| e.holder != who)
    }    
    fn write_u8(&self, addr: &u64, data: &u8) -> Result<(), u64> {
        self.space.borrow().write_bytes(addr, unsafe {
            std::slice::from_raw_parts(data as *const u8, 1)
        })?;
        Ok(())
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn read_u8(&self, addr: &u64, data: &mut u8) -> Result<(), u64> {
        self.space.borrow().read_bytes(addr, unsafe {
            std::slice::from_raw_parts_mut(data as *mut u8, 1)
        })?;
        Ok(())
    }

    fn write_u16(&self, addr: &u64, data: &u16) -> Result<(), u64> {
        self.space.borrow().write_bytes(addr, unsafe {
            std::slice::from_raw_parts((data as *const u16) as *const u8, 2)
        })?;
        Ok(())
    }

    fn read_u16(&self, addr: &u64, data: &mut u16) -> Result<(), u64> {
        self.space.borrow().read_bytes(addr, unsafe {
            std::slice::from_raw_parts_mut((data as *mut u16) as *mut u8, 2)
        })?;
        Ok(())
    }

    fn write_u32(&self, addr: &u64, data: &u32) -> Result<(), u64> {
        self.space.borrow().write_bytes(addr, unsafe {
            std::slice::from_raw_parts((data as *const u32) as *const u8, 4)
        })?;
        Ok(())
    }

    fn read_u32(&self, addr: &u64, data: &mut u32) -> Result<(), u64> {
        self.space.borrow().read_bytes(addr, unsafe {
            std::slice::from_raw_parts_mut((data as *mut u32) as *mut u8, 4)
        })?;
        Ok(())
    }

    fn write_u64(&self, addr: &u64, data: &u64) -> Result<(), u64> {
        self.space.borrow().write_bytes(addr, unsafe {
            std::slice::from_raw_parts((data as *const u64) as *const u8, 8)
        })?;
        Ok(())
    }

    fn read_u64(&self, addr: &u64, data: &mut u64) -> Result<(), u64> {
        self.space.borrow().read_bytes(addr, unsafe {
            std::slice::from_raw_parts_mut((data as *mut u64) as *mut u8, 8)
        })?;
        Ok(())
    }
}