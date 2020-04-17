use terminus_spaceport::memory::region::{Region, BytesAccess, U8Access, U16Access, U32Access, U64Access, IOAccess};
use terminus_spaceport::memory::{MemInfo, region};
use terminus_spaceport::space::{Space, SPACE_TABLE};
use terminus_spaceport::space;
use terminus_spaceport::derive_io;
use std::sync::{Arc, Mutex};
use std::fmt;
use super::elf::ElfLoader;
use std::ops::Deref;
use super::devices::htif::HTIF;
use std::fmt::{Display, Formatter};
use crate::processor::{ProcessorCfg, Processor};
use std::cmp::min;
use crate::devices::clint::Timer;

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


pub struct System {
    name: String,
    mem_space: Arc<Space>,
    bus: Arc<Bus>,
    timer: Arc<Timer>,
    elf: ElfLoader,
    processors: Vec<Processor>,
}

impl System {
    pub fn new(name: &str, elf_file: &str, processor_cfgs: Vec<ProcessorCfg>, timer_freq: usize) -> System {
        let space = SPACE_TABLE.get_space(name);
        let bus = Arc::new(Bus::new(&space));
        let elf = ElfLoader::new(elf_file).expect(&format!("Invalid Elf {}", elf_file));
        let mut sys = System {
            name: name.to_string(),
            mem_space: space,
            bus,
            timer: Arc::new(Timer::new(timer_freq)),
            elf,
            processors: vec![],
        };
        sys.try_register_htif();
        for cfg in processor_cfgs {
            sys.new_processor(cfg)
        }
        sys
    }

    fn new_processor(&mut self, config: ProcessorCfg) {
        let p = Processor::new(self.processors.len(), self.elf.entry_point().unwrap(), config, &self.bus, &self.timer().alloc_irq());
        self.processors.push(p)
    }

    fn register_region(&self, name: &str, base: u64, region: &Arc<Region>) -> Result<Arc<Region>, space::Error> {
        self.mem_space.add_region(name, &Region::remap(base, &region))
    }

    fn try_register_htif(&self) {
        if let Some(s) = self.elf.htif_section().expect("Invalid ELF!") {
            self.register_region("htif", s.address(), &Region::io(0, 0x1000, Box::new(HTIF::new()))).unwrap();
        }
    }

    pub fn processor(&self, hartid: usize) -> Option<&Processor> {
        if hartid >= self.processors.len() {
            None
        } else {
            Some(&self.processors[hartid])
        }
    }

    pub fn bus(&self) -> &Arc<Bus> {
        &self.bus
    }

    pub fn timer(&self) -> &Arc<Timer> {
        &self.timer
    }

    pub fn mem_space(&self) -> &Arc<Space> {
        &self.mem_space
    }

    pub fn register_device<D: IOAccess + 'static>(&self, name: &str, base: u64, size: u64, device: D) -> Result<Arc<Region>, space::Error> {
        self.register_region(name, base, &Region::io(base, size, Box::new(device)))
    }

    pub fn register_memory(&self, name: &str, base: u64, mem: &Arc<Region>) {
        match self.register_region(name, base, &mem) {
            Ok(_) => {}
            Err(space::Error::Overlap(n, msg)) => {
                if n == "htif".to_string() {
                    let htif_region = self.mem_space.get_region(&n).unwrap();
                    let range0 = if base < htif_region.info.base {
                        Some(MemInfo { base: base, size: htif_region.info.base - base })
                    } else {
                        None
                    };
                    let range1 = if base + mem.info.size > htif_region.info.base + htif_region.info.size {
                        Some(MemInfo { base: htif_region.info.base + htif_region.info.size, size: base + mem.info.size - (htif_region.info.base + htif_region.info.size) })
                    } else {
                        None
                    };
                    range0.iter().for_each(|info| {
                        self.register_region(&format!("{}_0", name), info.base, &Region::remap_partial(0, mem, 0, info.size)).unwrap();
                    });
                    range1.iter().for_each(|info| {
                        self.register_region(&format!("{}_1", name), info.base, &Region::remap_partial(0, mem, info.base - base, info.size)).unwrap();
                    });
                } else {
                    panic!(msg)
                }
            }
            Err(space::Error::Renamed(_, msg)) => panic!(msg)
        }
    }


    pub fn load_elf(&self) {
        self.elf.load(|addr, data| {
            fn load(space: &Space, addr: u64, data: &[u8]) -> Result<(), String> {
                if data.is_empty() {
                    Ok(())
                } else {
                    if let Ok(ref region) = space.get_region_by_addr(addr) {
                        let len = min((region.info.base + region.info.size - addr) as usize, data.len());
                        let (head, tails) = data.split_at(len);
                        if let Err(e) = BytesAccess::write(region.deref(), addr, head) {
                            return Err(format!("{:?}", e));
                        }
                        load(space, region.info.base + region.info.size, tails)
                    } else {
                        Err(format!("not enough memory!"))
                    }
                }
            };
            load(self.mem_space().deref(), addr, data)
        }).expect(&format!("{} load elf fail!", self.name));
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Machine {}:", self.name)?;
        writeln!(f, "   {}", self.mem_space.to_string())
    }
}


