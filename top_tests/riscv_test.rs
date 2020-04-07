use terminus::processor::{ProcessorCfg, PrivilegeLevel};
use terminus::system::{System, SimCmd, SimResp};
use terminus_global::XLen;
use terminus_spaceport::memory::region::{GHEAP, U64Access};
use terminus_spaceport::devices::term_exit;
use std::ops::Deref;

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

    }).unwrap();

    loop {
        let resp = sys.sim_controller().send_cmd(0, SimCmd::RunOne);
        if let Ok(SimResp::Exited(msg, resp)) = resp {
            println!("{}:", msg);
            println!("{}",  resp.to_string());
            let htif = sys.mem_space().get_region("htif").unwrap();
            assert_eq!(U64Access::read(htif.deref(),htif.info.base).unwrap(), 0x1);
            break;
        } else if let Ok(SimResp::Resp(resp)) = resp {
            println!("{}", resp.trace());
        }
    }
    p0.join().unwrap();

    term_exit()
}

