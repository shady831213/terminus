use terminus::processor::{ProcessorCfg, PrivilegeLevel};
use terminus::system::System;
use terminus_global::XLen;
use terminus_spaceport::memory::region::{GHEAP, U64Access};
use terminus_spaceport::devices::term_exit;
use std::ops::Deref;
use std::path::Path;
use terminus_spaceport::EXIT_CTRL;
use terminus::devices::clint::Clint;

fn riscv_test(xlen: XLen, name: &str, debug: bool) -> bool {
    EXIT_CTRL.reset();
    let processor_cfg = ProcessorCfg {
        xlen,
        privilege_level: PrivilegeLevel::MSU,
        enable_dirty: true,
        extensions: vec![].into_boxed_slice(),
    };
    let sys = System::new(name, Path::new("top_tests/elf").join(Path::new(name)).to_str().expect(&format!("{} not existed!", name)), vec![processor_cfg], 100);
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x10000000, 1).expect("main_memory alloc fail!"));
    sys.register_memory("rom", 0x20000000, &GHEAP.alloc(0x10000000, 1).expect("rom alloc fail!"));
    sys.register_device("clint", 0x20000, 0x10000, Clint::new(sys.timer())).unwrap();
    sys.load_elf();


    let p = sys.processor(0).unwrap();

    let interval: u64 = 100;
    let mut interval_cnt: u64 = 0;
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            if debug {
                println!("{}", msg)
            }
            break;
        }
        p.step(1);
        interval_cnt += 1;
        if debug {
            println!("{}", p.state().trace())
        }
        if interval_cnt % interval == interval - 1 {
            sys.timer().tick(interval)
        }
    }
    if debug {
        println!("{}", p.state().to_string())
    }

    let htif = sys.mem_space().get_region("htif").unwrap();
    U64Access::read(htif.deref(), htif.info.base).unwrap() == 0x1
}

fn main() {
    let mut args = std::env::args();
    let mut debug = false;
    let mut name: Option<String> = None;
    let mut arg: Option<String> = args.next();
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
    //ui-p
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
    riscv_test!(XLen::X64, "rv64ui-p-slli");
    riscv_test!(XLen::X64, "rv64ui-p-slliw");
    riscv_test!(XLen::X64, "rv64ui-p-sllw");
    riscv_test!(XLen::X64, "rv64ui-p-slt");
    riscv_test!(XLen::X64, "rv64ui-p-slti");
    riscv_test!(XLen::X64, "rv64ui-p-sltiu");
    riscv_test!(XLen::X64, "rv64ui-p-sltu");
    riscv_test!(XLen::X64, "rv64ui-p-sra");
    riscv_test!(XLen::X64, "rv64ui-p-srai");
    riscv_test!(XLen::X64, "rv64ui-p-sraiw");
    riscv_test!(XLen::X64, "rv64ui-p-sraw");
    riscv_test!(XLen::X64, "rv64ui-p-srl");
    riscv_test!(XLen::X64, "rv64ui-p-srli");
    riscv_test!(XLen::X64, "rv64ui-p-srliw");
    riscv_test!(XLen::X64, "rv64ui-p-srlw");
    riscv_test!(XLen::X64, "rv64ui-p-sub");
    riscv_test!(XLen::X64, "rv64ui-p-subw");
    riscv_test!(XLen::X64, "rv64ui-p-xor");
    riscv_test!(XLen::X64, "rv64ui-p-xori");
    riscv_test!(XLen::X32, "rv32ui-p-add");
    riscv_test!(XLen::X32, "rv32ui-p-addi");
    riscv_test!(XLen::X32, "rv32ui-p-and");
    riscv_test!(XLen::X32, "rv32ui-p-andi");
    riscv_test!(XLen::X32, "rv32ui-p-auipc");
    riscv_test!(XLen::X32, "rv32ui-p-beq");
    riscv_test!(XLen::X32, "rv32ui-p-bge");
    riscv_test!(XLen::X32, "rv32ui-p-bgeu");
    riscv_test!(XLen::X32, "rv32ui-p-blt");
    riscv_test!(XLen::X32, "rv32ui-p-bltu");
    riscv_test!(XLen::X32, "rv32ui-p-bne");
    riscv_test!(XLen::X32, "rv32ui-p-fence_i");
    riscv_test!(XLen::X32, "rv32ui-p-jal");
    riscv_test!(XLen::X32, "rv32ui-p-jalr");
    riscv_test!(XLen::X32, "rv32ui-p-lb");
    riscv_test!(XLen::X32, "rv32ui-p-lbu");
    riscv_test!(XLen::X32, "rv32ui-p-lh");
    riscv_test!(XLen::X32, "rv32ui-p-lhu");
    riscv_test!(XLen::X32, "rv32ui-p-lui");
    riscv_test!(XLen::X32, "rv32ui-p-lw");
    riscv_test!(XLen::X32, "rv32ui-p-or");
    riscv_test!(XLen::X32, "rv32ui-p-ori");
    riscv_test!(XLen::X32, "rv32ui-p-sb");
    riscv_test!(XLen::X32, "rv32ui-p-sh");
    riscv_test!(XLen::X32, "rv32ui-p-simple");
    riscv_test!(XLen::X32, "rv32ui-p-sll");
    riscv_test!(XLen::X32, "rv32ui-p-slli");
    riscv_test!(XLen::X32, "rv32ui-p-slt");
    riscv_test!(XLen::X32, "rv32ui-p-slti");
    riscv_test!(XLen::X32, "rv32ui-p-sltiu");
    riscv_test!(XLen::X32, "rv32ui-p-sltu");
    riscv_test!(XLen::X32, "rv32ui-p-sra");
    riscv_test!(XLen::X32, "rv32ui-p-srai");
    riscv_test!(XLen::X32, "rv32ui-p-srl");
    riscv_test!(XLen::X32, "rv32ui-p-srli");
    riscv_test!(XLen::X32, "rv32ui-p-sub");
    riscv_test!(XLen::X32, "rv32ui-p-sw");
    riscv_test!(XLen::X32, "rv32ui-p-xor");
    riscv_test!(XLen::X32, "rv32ui-p-xori");

    //mi-p-*
    riscv_test!(XLen::X64, "rv64mi-p-access");
    //no debug
    //riscv_test!(XLen::X64, "rv64mi-p-breakpoint");
    //riscv_test!(XLen::X64, "rv64mi-p-sbreak");
    riscv_test!(XLen::X64, "rv64mi-p-csr");
    //no interrupt
    riscv_test!(XLen::X64, "rv64mi-p-illegal");
    riscv_test!(XLen::X64, "rv64mi-p-ma_addr");
    riscv_test!(XLen::X64, "rv64mi-p-ma_fetch");
    riscv_test!(XLen::X64, "rv64mi-p-mcsr");
    riscv_test!(XLen::X64, "rv64mi-p-scall");

    //no debug
    //riscv_test!(XLen::X32, "rv32mi-p-breakpoint");
    //riscv_test!(XLen::X32, "rv32mi-p-sbreak");
    riscv_test!(XLen::X32, "rv32mi-p-csr");
    //no interrupt
    //riscv_test!(XLen::X32, "rv32mi-p-illegal");
    riscv_test!(XLen::X32, "rv32mi-p-ma_addr");
    riscv_test!(XLen::X32, "rv32mi-p-ma_fetch");
    riscv_test!(XLen::X32, "rv32mi-p-mcsr");
    riscv_test!(XLen::X32, "rv32mi-p-scall");
    riscv_test!(XLen::X32, "rv32mi-p-shamt");

    //si-p-*
    riscv_test!(XLen::X64, "rv64si-p-csr");
    riscv_test!(XLen::X64, "rv64si-p-dirty");
    riscv_test!(XLen::X64, "rv64si-p-ma_fetch");
    //no debug
    //riscv_test!(XLen::X64, "rv64si-p-sbreak");
    riscv_test!(XLen::X64, "rv64si-p-scall");
    //no interrupt
    //riscv_test!(XLen::X64, "rv64si-p-wfi");

    riscv_test!(XLen::X32, "rv32si-p-csr");
    riscv_test!(XLen::X32, "rv32si-p-dirty");
    riscv_test!(XLen::X32, "rv32si-p-ma_fetch");
    //no debug
    //riscv_test!(XLen::X32, "rv32si-p-sbreak");
    riscv_test!(XLen::X32, "rv32si-p-scall");
    //no interrupt
    //riscv_test!(XLen::X32, "rv32si-p-wfi");

    term_exit()
}


