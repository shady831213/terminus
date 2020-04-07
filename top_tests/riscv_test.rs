use terminus::processor::{Processor, ProcessorCfg, PrivilegeLevel};
use terminus::system::{System, SimCmd};
use terminus_global::XLen;
use terminus_spaceport::memory::region::GHEAP;
use terminus_spaceport::devices::term_exit;

#[test]
fn riscv_basic_test() {
    let sys = System::new("m0", "top_tests/elf/rv64ui-p-add");
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x10000000, 1).expect("main_memory alloc fail!"));
    sys.register_memory("rom", 0x20000000, &GHEAP.alloc(0x10000000, 1).expect("rom alloc fail!"));
    sys.load_elf();

    let processor_cfg = ProcessorCfg {
        xlen: XLen::X64,
        hartid: 0,
        start_address: sys.entry_point().expect("Invalid ELF!"),
        privilege_level: PrivilegeLevel::MSU,
        enable_dirty: true,
        extensions: vec![].into_boxed_slice(),
    };

    let p0 = sys.new_processor("p0", processor_cfg, |p| {
        p.run().unwrap();
        println!("{}", p.state().to_string());
    }).unwrap();

    sys.sim_controller().send_cmd(0, SimCmd::RunAll).unwrap();
    p0.join().unwrap();

    term_exit()
}

