use terminus_spaceport::memory::MemInfo;
use terminus_spaceport::space::{Space, SPACE_TABLE};
use terminus_spaceport::space;
use terminus_spaceport::memory::region::{Region,IOAccess,BytesAccess};
use std::sync::Arc;
use std::fmt;
use crate::devices::htif::HTIF;
use crate::devices::bus::Bus;
use std::fmt::{Display, Formatter};
use crate::processor::{ProcessorCfg, Processor};
use std::cmp::min;
use crate::devices::clint::{Timer, Clint};
use std::ops::Deref;

pub mod elf;
use elf::ElfLoader;

pub mod fdt;

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

    fn register_region(&self, name: &str, base: u64, region: &Arc<Region>) -> Result<(), space::Error> {
        self.mem_space.add_region(name, &Region::remap(base, &region))?;
        Ok(())
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

    pub fn processors(&self) -> &Vec<Processor> {
        &self.processors
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

    pub fn register_device<D: IOAccess + 'static>(&self, name: &str, base: u64, size: u64, device: D) -> Result<(), space::Error> {
        self.register_region(name, base, &Region::io(base, size, Box::new(device)))
    }


    pub fn register_memory(&self, name: &str, base: u64, mem: &Arc<Region>)-> Result<(), space::Error> {
        match self.register_region(name, base, &mem) {
            Ok(_) => {Ok(())}
            Err(e) => {
                if let space::Error::Overlap(n, msg) = e {
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
                            self.register_region(name, info.base, &Region::remap_partial(0, mem, 0, info.size)).unwrap();
                        });
                        range1.iter().for_each(|info| {
                            self.register_region(&format!("{}_1", name), info.base, &Region::remap_partial(0, mem, info.base - base, info.size)).unwrap();
                        });
                        Ok(())
                    } else {
                        Err(space::Error::Overlap(n, msg))
                    }
                } else {
                    Err(e)
                }
            }
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


