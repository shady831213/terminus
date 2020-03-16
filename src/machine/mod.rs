use dpi_memory::{Space, SpaceTable, MemInfo, Heap, Region};
use std::sync::{Arc, RwLock};
use std::ops::Deref;
use xmas_elf::ElfFile;
use xmas_elf::header::Header;
use xmas_elf::header;
use std::fs;

pub struct Machine {
    name: String,
    mem_space: Arc<RwLock<Space>>,
}

impl Machine {
    pub fn new(name: &str) -> Machine {
        Machine {
            name: name.to_string(),
            mem_space: SpaceTable::global().get_space(name),
        }
    }

    pub fn register_mem(&self, name: &str, info: MemInfo) {
        let mem = Heap::global().alloc(info.size, 1);
        self.mem_space.write().unwrap().add_region(name, &Region::mmap(info.base, &mem));
    }
}

#[test]
fn machine_basic() {
    let m = Machine::new("m0");
    m.register_mem("main_memory", MemInfo { base: 0x80000000, size: 0x10000000 });
    m.register_mem("rom", MemInfo { base: 0x20000000, size: 0x10000000 });
}

struct ElfLoader<'a> {
    elf: ElfFile<'a>,
}

impl<'a> ElfLoader<'a> {
    pub fn new(input: &'a [u8]) -> Result<ElfLoader<'a>, String> {
        let elf = ElfFile::new(input)?;
        Ok(ElfLoader {
            elf
        })
    }

    fn check_header(&self) -> Result<Header, String> {
        //check riscv
        if let header::Machine::Other(id) = self.elf.header.pt2.machine().as_machine() {
            if id == 243 {
                Ok(self.elf.header)
            } else {
                Err(format!("Invalid Arch {:?}!", self.elf.header.pt2.machine()))
            }
        } else {
            Err(format!("Invalid Arch {:?}!", self.elf.header.pt2.machine()))
        }
    }
}