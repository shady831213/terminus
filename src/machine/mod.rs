use dpi_memory::{Space, SpaceTable, Heap, Region, BytesAccess};
use std::sync::{Arc, RwLock};
use std::fs;
use super::elf::ElfLoader;
use std::ops::Deref;
use super::devices::htif::HTIF;

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

    pub fn register_region(&self, name: &str, base: u64, region: &Arc<Region>) {
        self.mem_space.write().unwrap().add_region(name, &Region::mmap(base, &region));
    }

    pub fn load_elf(&self, file_path: &str) {
        let blob = fs::read(file_path).expect("Can't read binary");
        let elf = ElfLoader::new(blob.as_slice()).expect(&format!("Invalid ELF {}!", file_path));
        elf.load(|name, addr, data| {
            let region = self.mem_space.read().unwrap().get_region_by_addr(addr);
            if addr + data.len() as u64 > region.info.base + region.info.size {
                Err(format!("section {} not enough memory!", name))
            } else {
                Ok(BytesAccess::write(region.deref(), addr, data))
            }
        }).expect(&format!("{} load fail!", file_path));
    }
}

#[test]
fn machine_basic() {
    let m = Machine::new("m0");
    m.register_region("main_memory", 0x80000000, &Heap::global().alloc(0x1000, 1));
    m.register_region("rom", 0x20000000, &Heap::global().alloc(0x1000, 1));
    m.register_region("htif", 0x80001000, &Region::io(0, 0x1000, Box::new(HTIF::new())));
    m.load_elf("top_tests/elf/rv64ui-p-add");
}

