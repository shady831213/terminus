use terminus::processor::{Processor, ProcessorCfg, PrivilegeLevel};
use terminus::system::System;
use terminus_global::XLen;
use terminus_spaceport::memory::region::GHEAP;
use terminus::elf::ElfLoader;
use terminus_spaceport::EXIT_CTRL;
use terminus_spaceport::devices::term_exit;

#[test]
fn riscv_basic_test() {
    let sys = System::new("m0");
    let elf = ElfLoader::new("top_tests/elf/rv64ui-p-add").expect("Invalid ELF {}!");
    sys.try_register_htif(&elf);
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x10000000, 1).expect("main_memory alloc fail!"));
    sys.register_memory("rom", 0x20000000, &GHEAP.alloc(0x10000000, 1).expect("rom alloc fail!"));
    sys.load_elf(&elf);

    let processor_cfg = ProcessorCfg {
        xlen: XLen::X64,
        hartid: 0,
        start_address: elf.entry_point().expect("Invalid ELF!"),
        privilege_level: PrivilegeLevel::MSU,
        enabel_dirty: true,
    };

    let p = Processor::new(processor_cfg, sys.mem_space(), vec![]);
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            println!("{}", msg);
            break
        }
        p.step_one();
        println!("{}", p.state().trace());

    }
    println!("{}", p.state().to_string());
    term_exit()
}

