use terminus::processor::{ProcessorCfg, PrivilegeLevel};
use terminus::system::{System, SimCmd, SimResp};
use terminus_global::XLen;
use terminus_spaceport::memory::region::{GHEAP, U64Access};
use terminus_spaceport::devices::term_exit;
use std::ops::Deref;
use std::path::Path;
use terminus_spaceport::EXIT_CTRL;

fn riscv_test(xlen: XLen, name: &str, debug: bool) -> bool {
    EXIT_CTRL.reset();
    let sys = System::new(name, Path::new("top_tests/elf").join(Path::new(name)).to_str().expect(&format!("{} not existed!", name)));
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
            if debug {
                println!("{}:", msg);
                println!("{}", resp.to_string());
            }
            break;
        } else if let Ok(SimResp::Resp(resp)) = resp {
            if debug {
                println!("{}", resp.trace());
            }
        } else if resp.is_err() {
            break
        }
    }
    p0.join().unwrap();

    let htif = sys.mem_space().get_region("htif").unwrap();
    U64Access::read(htif.deref(), htif.info.base).unwrap() == 0x1
}

fn main() {
    let mut args = std::env::args();
    let mut debug = false;
    let mut name:Option<String> = None;
    let mut arg:Option<String> = args.next();
    while let Some(a) = &arg {
        if *a == "-d".to_string() {
            debug = true
        }
        if *a == "-r".to_string() {
            name = args.next()
        }
        arg = args.next()
    }

    macro_rules! riscv_test {
        ($xlen:expr, $name:expr) => {
            if let Some(test_name) = &name {
                if test_name == $name {
                    if !riscv_test($xlen, $name, debug) {
                        term_exit();
                        assert!(false,format!("{} fail!",$name))
                    }
                    println!("{}", format!("{} pass!",$name));
                }
            } else {
                if !riscv_test($xlen, $name, debug) {
                    term_exit();
                    assert!(false,format!("{} fail!",$name))
                }
                println!("{}", format!("{} pass!",$name));
            }
        };
    }

    riscv_test!(XLen::X64, "rv64ui-p-add");
    riscv_test!(XLen::X64, "rv64ui-p-addi");
    riscv_test!(XLen::X64, "rv64ui-p-addiw");
    riscv_test!(XLen::X64, "rv64ui-p-addw");
    riscv_test!(XLen::X64, "rv64ui-p-and");
    riscv_test!(XLen::X64, "rv64ui-p-andi");
    riscv_test!(XLen::X64, "rv64ui-p-auipc");
    riscv_test!(XLen::X64, "rv64ui-p-beq");
    riscv_test!(XLen::X64, "rv64ui-p-bge");
    riscv_test!(XLen::X64, "rv64ui-p-bgeu");
    riscv_test!(XLen::X64, "rv64ui-p-blt");
    riscv_test!(XLen::X64, "rv64ui-p-bltu");
    riscv_test!(XLen::X64, "rv64ui-p-bne");
    riscv_test!(XLen::X64, "rv64ui-p-fence_i");
    riscv_test!(XLen::X64, "rv64ui-p-jal");
    riscv_test!(XLen::X64, "rv64ui-p-jalr");
    riscv_test!(XLen::X64, "rv64ui-p-lb");
    riscv_test!(XLen::X64, "rv64ui-p-lbu");
    riscv_test!(XLen::X64, "rv64ui-p-ld");
    riscv_test!(XLen::X64, "rv64ui-p-lh");
    riscv_test!(XLen::X64, "rv64ui-p-lhu");
    riscv_test!(XLen::X64, "rv64ui-p-lui");
    riscv_test!(XLen::X64, "rv64ui-p-lw");
    riscv_test!(XLen::X64, "rv64ui-p-lwu");
    riscv_test!(XLen::X64, "rv64ui-p-or");
    riscv_test!(XLen::X64, "rv64ui-p-ori");
    riscv_test!(XLen::X64, "rv64ui-p-sb");
    riscv_test!(XLen::X64, "rv64ui-p-sd");
    riscv_test!(XLen::X64, "rv64ui-p-sh");
    riscv_test!(XLen::X64, "rv64ui-p-simple");
    riscv_test!(XLen::X64, "rv64ui-p-sll");
    riscv_test!(XLen::X32, "rv32ui-p-add");
    term_exit()
}


