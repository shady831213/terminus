extern crate rand;

use terminus::processor::ProcessorCfg;
use terminus::system::System;
use terminus_global::XLen;
use terminus_spaceport::memory::region::{GHEAP, U64Access};
use terminus_spaceport::devices::term_exit;
use std::ops::Deref;
use std::path::Path;
use terminus_spaceport::EXIT_CTRL;
use terminus::devices::clint::Clint;
use rand::thread_rng;
use rand::seq::SliceRandom;

fn riscv_test(xlen: XLen, name: &str, debug: bool, num_cores: usize) -> bool {
    EXIT_CTRL.reset();
    let configs = vec![ProcessorCfg {
        xlen,
        enable_dirty: true,
        extensions: vec!['m', 'f', 'd', 's', 'u', 'c', 'a'].into_boxed_slice(),
    }; num_cores];
    let sys = System::new(name, Path::new("top_tests/elf").join(Path::new(name)).to_str().expect(&format!("{} not existed!", name)), configs, 100);
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x10000000, 1).expect("main_memory alloc fail!"));
    sys.register_memory("rom", 0x20000000, &GHEAP.alloc(0x10000000, 1).expect("rom alloc fail!"));
    sys.register_device("clint", 0x20000, 0x10000, Clint::new(sys.timer())).unwrap();
    sys.load_elf();

    let mut cores = vec![];
    for p in sys.processors() {
        cores.push(p)
    }
    let mut rng = thread_rng();

    let interval: u64 = 100;
    let mut interval_cnt: u64 = 0;
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            if debug {
                println!("{}", msg)
            }
            break;
        }
        cores.shuffle(&mut rng);
        for p in &cores {
            p.step(1);
            if debug {
                println!("{}", p.state().trace())
            }
        }
        interval_cnt += 1;
        if interval_cnt % interval == interval - 1 {
            sys.timer().tick(interval)
        }
    }
    if debug {
        for p in sys.processors() {
            println!("{}", p.state().to_string())
        }
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
                    if !riscv_test($xlen, $name, debug, 1) {
                        term_exit();
                        assert!(false,format!("{} fail!",$name))
                    }
                    println!("{}", format!("{} pass!",$name));
                }
            } else {
                if !riscv_test($xlen, $name, debug, 1) {
                    term_exit();
                    assert!(false,format!("{} fail!",$name))
                }
                println!("{}", format!("{} pass!",$name));
            }
        };
    }
    macro_rules! riscv_test_mp {
        ($xlen:expr, $name:expr, $num:expr) => {
            if let Some(test_name) = &name {
                if test_name == $name {
                    if !riscv_test($xlen, $name, debug, $num) {
                        term_exit();
                        assert!(false,format!("{} fail!",$name))
                    }
                    println!("{}", format!("{} pass!",$name));
                }
            } else {
                if !riscv_test($xlen, $name, debug, $num) {
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
    riscv_test!(XLen::X64, "rv64mi-p-breakpoint");
    riscv_test!(XLen::X64, "rv64mi-p-sbreak");
    riscv_test!(XLen::X64, "rv64mi-p-csr");
    riscv_test!(XLen::X64, "rv64mi-p-illegal");
    riscv_test!(XLen::X64, "rv64mi-p-ma_addr");
    riscv_test!(XLen::X64, "rv64mi-p-ma_fetch");
    riscv_test!(XLen::X64, "rv64mi-p-mcsr");
    riscv_test!(XLen::X64, "rv64mi-p-scall");

    riscv_test!(XLen::X32, "rv32mi-p-breakpoint");
    riscv_test!(XLen::X32, "rv32mi-p-sbreak");
    riscv_test!(XLen::X32, "rv32mi-p-csr");
    riscv_test!(XLen::X32, "rv32mi-p-illegal");
    riscv_test!(XLen::X32, "rv32mi-p-ma_addr");
    riscv_test!(XLen::X32, "rv32mi-p-ma_fetch");
    riscv_test!(XLen::X32, "rv32mi-p-mcsr");
    riscv_test!(XLen::X32, "rv32mi-p-scall");
    riscv_test!(XLen::X32, "rv32mi-p-shamt");

    //si-p-*
    riscv_test!(XLen::X64, "rv64si-p-csr");
    riscv_test!(XLen::X64, "rv64si-p-dirty");
    riscv_test!(XLen::X64, "rv64si-p-ma_fetch");
    riscv_test!(XLen::X64, "rv64si-p-sbreak");
    riscv_test!(XLen::X64, "rv64si-p-scall");
    riscv_test!(XLen::X64, "rv64si-p-wfi");

    riscv_test!(XLen::X32, "rv32si-p-csr");
    riscv_test!(XLen::X32, "rv32si-p-dirty");
    riscv_test!(XLen::X32, "rv32si-p-ma_fetch");
    riscv_test!(XLen::X32, "rv32si-p-sbreak");
    riscv_test!(XLen::X32, "rv32si-p-scall");
    riscv_test!(XLen::X32, "rv32si-p-wfi");

    //um-p-*
    riscv_test!(XLen::X64, "rv64um-p-mul");
    riscv_test!(XLen::X64, "rv64um-p-mulh");
    riscv_test!(XLen::X64, "rv64um-p-mulhsu");
    riscv_test!(XLen::X64, "rv64um-p-mulhu");
    riscv_test!(XLen::X64, "rv64um-p-mulw");
    riscv_test!(XLen::X64, "rv64um-p-div");
    riscv_test!(XLen::X64, "rv64um-p-divu");
    riscv_test!(XLen::X64, "rv64um-p-divw");
    riscv_test!(XLen::X64, "rv64um-p-divuw");
    riscv_test!(XLen::X64, "rv64um-p-rem");
    riscv_test!(XLen::X64, "rv64um-p-remu");
    riscv_test!(XLen::X64, "rv64um-p-remw");
    riscv_test!(XLen::X64, "rv64um-p-remuw");

    riscv_test!(XLen::X32, "rv32um-p-mul");
    riscv_test!(XLen::X32, "rv32um-p-mulh");
    riscv_test!(XLen::X32, "rv32um-p-mulhsu");
    riscv_test!(XLen::X32, "rv32um-p-mulhu");
    riscv_test!(XLen::X32, "rv32um-p-div");
    riscv_test!(XLen::X32, "rv32um-p-divu");
    riscv_test!(XLen::X32, "rv32um-p-rem");
    riscv_test!(XLen::X32, "rv32um-p-remu");

    //uf-p-*
    riscv_test!(XLen::X64, "rv64uf-p-fadd");
    riscv_test!(XLen::X64, "rv64uf-p-fclass");
    riscv_test!(XLen::X64, "rv64uf-p-fcmp");
    riscv_test!(XLen::X64, "rv64uf-p-fcvt");
    riscv_test!(XLen::X64, "rv64uf-p-fcvt_w");
    riscv_test!(XLen::X64, "rv64uf-p-fdiv");
    riscv_test!(XLen::X64, "rv64uf-p-fmadd");
    riscv_test!(XLen::X64, "rv64uf-p-fmin");
    riscv_test!(XLen::X64, "rv64uf-p-ldst");
    riscv_test!(XLen::X64, "rv64uf-p-move");
    riscv_test!(XLen::X64, "rv64uf-p-recoding");

    riscv_test!(XLen::X32, "rv32uf-p-fadd");
    riscv_test!(XLen::X32, "rv32uf-p-fclass");
    riscv_test!(XLen::X32, "rv32uf-p-fcmp");
    riscv_test!(XLen::X32, "rv32uf-p-fcvt");
    riscv_test!(XLen::X32, "rv32uf-p-fcvt_w");
    riscv_test!(XLen::X32, "rv32uf-p-fdiv");
    riscv_test!(XLen::X32, "rv32uf-p-fmadd");
    riscv_test!(XLen::X32, "rv32uf-p-fmin");
    riscv_test!(XLen::X32, "rv32uf-p-ldst");
    riscv_test!(XLen::X32, "rv32uf-p-move");
    riscv_test!(XLen::X32, "rv32uf-p-recoding");

    //ud-p-*
    riscv_test!(XLen::X64, "rv64ud-p-fadd");
    riscv_test!(XLen::X64, "rv64ud-p-fclass");
    riscv_test!(XLen::X64, "rv64ud-p-fcmp");
    riscv_test!(XLen::X64, "rv64ud-p-fcvt");
    riscv_test!(XLen::X64, "rv64ud-p-fcvt_w");
    riscv_test!(XLen::X64, "rv64ud-p-fdiv");
    riscv_test!(XLen::X64, "rv64ud-p-fmadd");
    riscv_test!(XLen::X64, "rv64ud-p-fmin");
    riscv_test!(XLen::X64, "rv64ud-p-ldst");
    riscv_test!(XLen::X64, "rv64ud-p-move");
    riscv_test!(XLen::X64, "rv64ud-p-recoding");
    riscv_test!(XLen::X64, "rv64ud-p-structural");

    riscv_test!(XLen::X32, "rv32ud-p-fadd");
    riscv_test!(XLen::X32, "rv32ud-p-fclass");
    riscv_test!(XLen::X32, "rv32ud-p-fcmp");
    riscv_test!(XLen::X32, "rv32ud-p-fcvt");
    riscv_test!(XLen::X32, "rv32ud-p-fcvt_w");
    riscv_test!(XLen::X32, "rv32ud-p-fdiv");
    riscv_test!(XLen::X32, "rv32ud-p-fmadd");
    riscv_test!(XLen::X32, "rv32ud-p-fmin");
    riscv_test!(XLen::X32, "rv32ud-p-ldst");
    riscv_test!(XLen::X32, "rv32ud-p-recoding");

    //uc-p-*
    riscv_test!(XLen::X64, "rv64uc-p-rvc");

    riscv_test!(XLen::X32, "rv32uc-p-rvc");

    //ua-p-*
    riscv_test!(XLen::X64, "rv64ua-p-amoadd_d");
    riscv_test!(XLen::X64, "rv64ua-p-amoadd_w");
    riscv_test!(XLen::X64, "rv64ua-p-amoand_d");
    riscv_test!(XLen::X64, "rv64ua-p-amoand_w");
    riscv_test!(XLen::X64, "rv64ua-p-amomax_d");
    riscv_test!(XLen::X64, "rv64ua-p-amomax_w");
    riscv_test!(XLen::X64, "rv64ua-p-amomaxu_d");
    riscv_test!(XLen::X64, "rv64ua-p-amomaxu_w");
    riscv_test!(XLen::X64, "rv64ua-p-amomin_d");
    riscv_test!(XLen::X64, "rv64ua-p-amomin_w");
    riscv_test!(XLen::X64, "rv64ua-p-amominu_d");
    riscv_test!(XLen::X64, "rv64ua-p-amominu_w");
    riscv_test!(XLen::X64, "rv64ua-p-amoor_d");
    riscv_test!(XLen::X64, "rv64ua-p-amoor_w");
    riscv_test!(XLen::X64, "rv64ua-p-amoswap_d");
    riscv_test!(XLen::X64, "rv64ua-p-amoswap_w");
    riscv_test!(XLen::X64, "rv64ua-p-amoxor_d");
    riscv_test!(XLen::X64, "rv64ua-p-amoxor_w");
    riscv_test_mp!(XLen::X64, "rv64ua-p-lrsc", 2);

    riscv_test!(XLen::X32, "rv32ua-p-amoadd_w");
    riscv_test!(XLen::X32, "rv32ua-p-amoand_w");
    riscv_test!(XLen::X32, "rv32ua-p-amomax_w");
    riscv_test!(XLen::X32, "rv32ua-p-amomaxu_w");
    riscv_test!(XLen::X32, "rv32ua-p-amomin_w");
    riscv_test!(XLen::X32, "rv32ua-p-amominu_w");
    riscv_test!(XLen::X32, "rv32ua-p-amoor_w");
    riscv_test!(XLen::X32, "rv32ua-p-amoswap_w");
    riscv_test!(XLen::X32, "rv32ua-p-amoxor_w");
    riscv_test_mp!(XLen::X32, "rv32ua-p-lrsc", 2);

    term_exit()
}


