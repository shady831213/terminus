use terminus::processor::{Processor, ProcessorCfg};
use terminus::system::System;
use terminus_global::XLen;
use terminus_spaceport::memory::region::GHEAP;
use std::fs;
use terminus::elf::ElfLoader;

#[test]
fn riscv_basic_test() {
    let sys = System::new("m0");
    let blob = fs::read("top_tests/elf/rv64ui-p-add").expect("Can't read binary");
    let elf = ElfLoader::new(blob.as_slice()).expect("Invalid ELF {}!");
    sys.try_register_htif(&elf);
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x10000000, 1).expect("main_memory alloc fail!"));
    sys.register_memory("rom", 0x20000000, &GHEAP.alloc(0x10000000, 1).expect("rom alloc fail!"));
    sys.load_elf(&elf);

    let processor_cfg = ProcessorCfg {
        xlen: XLen::X64,
        hartid: 0,
        start_address: elf.entry_point(),
        enabel_dirty: true,
    };

    let p = Processor::new(processor_cfg, sys.mem_space(), vec![]);
    p.execute_one().unwrap();
    p.execute_one().unwrap();
    println!("{:#x?}", p.execute_one());
    println!("{}", p.state().to_string())
}
