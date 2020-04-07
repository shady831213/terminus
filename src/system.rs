use terminus_spaceport::memory::region::{Region, BytesAccess};
use terminus_spaceport::memory::MemInfo;
use terminus_spaceport::space::{Space, SPACE_TABLE};
use terminus_spaceport::space;
use std::sync::Arc;
use std::fmt;
use super::elf::ElfLoader;
use std::ops::Deref;
use super::devices::htif::HTIF;
use std::fmt::{Display, Formatter};

pub struct System {
    name: String,
    mem_space: Arc<Space>,
}

impl System {
    pub fn new(name: &str) -> System {
        System {
            name: name.to_string(),
            mem_space: SPACE_TABLE.get_space(name),
        }
    }

    fn register_region(&self, name: &str, base: u64, region: &Arc<Region>) -> Result<Arc<Region>, space::Error> {
        self.mem_space.add_region(name, &Region::remap(base, &region))
    }

    pub fn try_register_htif(&self, elf: &ElfLoader) {
        if let Some(s) = elf.htif_section().expect("Invalid ELF!") {
            self.register_region("htif", s.address(), &Region::io(0, 0x1000, Box::new(HTIF::new()))).unwrap();
        }
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

    pub fn load_elf(&self, elf: &ElfLoader) {
        elf.load(|addr, data| {
            let region = self.mem_space.get_region_by_addr(addr).unwrap();
            if addr + data.len() as u64 > region.info.base + region.info.size {
                Err(format!("not enough memory!"))
            } else {
                if let Err(e) = BytesAccess::write(region.deref(), addr, data) {
                    Err(format!("{:?}", e))
                } else {
                    Ok(())
                }
            }
        }).expect(&format!("{} load elf fail!", self.name));
    }

    pub fn mem_space(&self) -> &Arc<Space> {
        &self.mem_space
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Machine {}:", self.name)?;
        writeln!(f, "   {}", self.mem_space.to_string())
    }
}

