use terminus::processor::{ProcessorCfg, PrivilegeLevel};
use terminus::system::{System, SimCmd, SimResp};
use terminus_global::XLen;
use terminus_spaceport::memory::region::{GHEAP, U64Access};
use terminus_spaceport::devices::term_exit;
use std::ops::Deref;
use std::path::Path;

fn riscv_test(xlen: XLen, name: &str) {
    let sys = System::new("m0", Path::new("top_tests/elf").join(Path::new(name)).to_str().expect(&format!("{} not existed!", name)));
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x10000000, 1).expect("main_memory alloc fail!"));
    sys.register_memory("rom", 0x20000000, &GHEAP.alloc(0x10000000, 1).expect("rom alloc fail!"));
    sys.load_elf();

    let processor_cfg = ProcessorCfg {
        xlen,
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
            println!("{}", resp.to_string());
            break;
        } else if let Ok(SimResp::Resp(resp)) = resp {
            println!("{}", resp.trace());
        }
    }
    p0.join().unwrap();

    term_exit();
    let htif = sys.mem_space().get_region("htif").unwrap();
    assert_eq!(U64Access::read(htif.deref(),htif.info.base).unwrap(), 0x1);
}

#[test]
fn rv64ui_p_add() {
    riscv_test(XLen::X64, "rv64ui-p-add")
}

#[test]
fn rv32ui_p_add() {
    riscv_test(XLen::X32, "rv32ui-p-add")
}


