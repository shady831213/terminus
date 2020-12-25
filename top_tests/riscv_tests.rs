use terminus::processor::ProcessorCfg;
use terminus::system::System;
use terminus::global::XLen;
use terminus_spaceport::memory::region::{GHEAP, U64Access};
use terminus_spaceport::devices::term_exit;
use std::ops::Deref;
use std::path::Path;
use terminus_spaceport::EXIT_CTRL;
use terminus::devices::clint::Clint;

struct RsicvTestRunner{
    debug: bool,
    tests_cnt:usize,
    name: Option<String>,
    timer:std::time::Instant,
}

impl RsicvTestRunner {
    pub fn new() -> RsicvTestRunner {
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
        RsicvTestRunner{
            debug,
            tests_cnt:0,
            name,
            timer:std::time::Instant::now(),
        }
    }

    fn test_mp(&mut self, xlen: XLen, name: &str,num_cores: usize) {
        let valid = {
            if let Some(test_name) = &self.name {
                test_name == name
            } else {
                true
            }
        };
        if valid {
            if !riscv_test(xlen, name, self.debug, num_cores) {
                term_exit();
                assert!(false, format!("{} fail!",name))
            }
            self.tests_cnt += 1;
            println!("{}", format!("{} pass!",name));
        }
    }

    fn test(&mut self, xlen: XLen, name: &str) {
        self.test_mp(xlen,name,1)
    }
}

impl Drop for RsicvTestRunner {
    fn drop(&mut self) {
        term_exit();
        println!("{} tests Pass in {} micro seconds!", self.tests_cnt, self.timer.elapsed().as_micros())
    }
}

fn riscv_test(xlen: XLen, name: &str, debug: bool, num_cores: usize) -> bool {
    EXIT_CTRL.reset();
    let configs = vec![ProcessorCfg {
        xlen,
        enable_dirty: true,
        extensions: vec!['m', 'f', 'd', 's', 'u', 'c', 'a'].into_boxed_slice(),
        freq:1000000000,
    }; num_cores];
    let mut sys = System::new(name, Path::new("top_tests/elf").join(Path::new(name)).to_str().expect(&format!("{} not existed!", name)), 10000000, 32);
    sys.register_htif(false);
    for cfg in configs {
        sys.new_processor(cfg)
    }
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x10000000, 1).expect("main_memory alloc fail!")).unwrap();
    sys.register_device("clint", 0x20000, 0x10000, Clint::new(sys.timer())).unwrap();
    sys.load_elf().unwrap();
    sys.reset(vec![-1i64 as u64;num_cores]).unwrap();

    let interval: u64 = 100;
    let mut interval_cnt: u64 = 0;
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            if debug {
                println!("{}", msg)
            }
            break;
        }
        for p in sys.processors() {
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
    let htif = sys.bus().space().get_region("htif").unwrap();
    U64Access::read(htif.deref(), &htif.info.base) == 0x1
}

fn main() {
    let mut runner = RsicvTestRunner::new();
    //ui-p
    runner.test(XLen::X64, "rv64ui-p-add");
    runner.test(XLen::X64, "rv64ui-p-addi");
    runner.test(XLen::X64, "rv64ui-p-addiw");
    runner.test(XLen::X64, "rv64ui-p-addw");
    runner.test(XLen::X64, "rv64ui-p-and");
    runner.test(XLen::X64, "rv64ui-p-andi");
    runner.test(XLen::X64, "rv64ui-p-auipc");
    runner.test(XLen::X64, "rv64ui-p-beq");
    runner.test(XLen::X64, "rv64ui-p-bge");
    runner.test(XLen::X64, "rv64ui-p-bgeu");
    runner.test(XLen::X64, "rv64ui-p-blt");
    runner.test(XLen::X64, "rv64ui-p-bltu");
    runner.test(XLen::X64, "rv64ui-p-bne");
    runner.test(XLen::X64, "rv64ui-p-fence_i");
    runner.test(XLen::X64, "rv64ui-p-jal");
    runner.test(XLen::X64, "rv64ui-p-jalr");
    runner.test(XLen::X64, "rv64ui-p-lb");
    runner.test(XLen::X64, "rv64ui-p-lbu");
    runner.test(XLen::X64, "rv64ui-p-ld");
    runner.test(XLen::X64, "rv64ui-p-lh");
    runner.test(XLen::X64, "rv64ui-p-lhu");
    runner.test(XLen::X64, "rv64ui-p-lui");
    runner.test(XLen::X64, "rv64ui-p-lw");
    runner.test(XLen::X64, "rv64ui-p-lwu");
    runner.test(XLen::X64, "rv64ui-p-or");
    runner.test(XLen::X64, "rv64ui-p-ori");
    runner.test(XLen::X64, "rv64ui-p-sb");
    runner.test(XLen::X64, "rv64ui-p-sd");
    runner.test(XLen::X64, "rv64ui-p-sh");
    runner.test(XLen::X64, "rv64ui-p-simple");
    runner.test(XLen::X64, "rv64ui-p-sll");
    runner.test(XLen::X64, "rv64ui-p-slli");
    runner.test(XLen::X64, "rv64ui-p-slliw");
    runner.test(XLen::X64, "rv64ui-p-sllw");
    runner.test(XLen::X64, "rv64ui-p-slt");
    runner.test(XLen::X64, "rv64ui-p-slti");
    runner.test(XLen::X64, "rv64ui-p-sltiu");
    runner.test(XLen::X64, "rv64ui-p-sltu");
    runner.test(XLen::X64, "rv64ui-p-sra");
    runner.test(XLen::X64, "rv64ui-p-srai");
    runner.test(XLen::X64, "rv64ui-p-sraiw");
    runner.test(XLen::X64, "rv64ui-p-sraw");
    runner.test(XLen::X64, "rv64ui-p-srl");
    runner.test(XLen::X64, "rv64ui-p-srli");
    runner.test(XLen::X64, "rv64ui-p-srliw");
    runner.test(XLen::X64, "rv64ui-p-srlw");
    runner.test(XLen::X64, "rv64ui-p-sub");
    runner.test(XLen::X64, "rv64ui-p-subw");
    runner.test(XLen::X64, "rv64ui-p-xor");
    runner.test(XLen::X64, "rv64ui-p-xori");
    runner.test(XLen::X32, "rv32ui-p-add");
    runner.test(XLen::X32, "rv32ui-p-addi");
    runner.test(XLen::X32, "rv32ui-p-and");
    runner.test(XLen::X32, "rv32ui-p-andi");
    runner.test(XLen::X32, "rv32ui-p-auipc");
    runner.test(XLen::X32, "rv32ui-p-beq");
    runner.test(XLen::X32, "rv32ui-p-bge");
    runner.test(XLen::X32, "rv32ui-p-bgeu");
    runner.test(XLen::X32, "rv32ui-p-blt");
    runner.test(XLen::X32, "rv32ui-p-bltu");
    runner.test(XLen::X32, "rv32ui-p-bne");
    runner.test(XLen::X32, "rv32ui-p-fence_i");
    runner.test(XLen::X32, "rv32ui-p-jal");
    runner.test(XLen::X32, "rv32ui-p-jalr");
    runner.test(XLen::X32, "rv32ui-p-lb");
    runner.test(XLen::X32, "rv32ui-p-lbu");
    runner.test(XLen::X32, "rv32ui-p-lh");
    runner.test(XLen::X32, "rv32ui-p-lhu");
    runner.test(XLen::X32, "rv32ui-p-lui");
    runner.test(XLen::X32, "rv32ui-p-lw");
    runner.test(XLen::X32, "rv32ui-p-or");
    runner.test(XLen::X32, "rv32ui-p-ori");
    runner.test(XLen::X32, "rv32ui-p-sb");
    runner.test(XLen::X32, "rv32ui-p-sh");
    runner.test(XLen::X32, "rv32ui-p-simple");
    runner.test(XLen::X32, "rv32ui-p-sll");
    runner.test(XLen::X32, "rv32ui-p-slli");
    runner.test(XLen::X32, "rv32ui-p-slt");
    runner.test(XLen::X32, "rv32ui-p-slti");
    runner.test(XLen::X32, "rv32ui-p-sltiu");
    runner.test(XLen::X32, "rv32ui-p-sltu");
    runner.test(XLen::X32, "rv32ui-p-sra");
    runner.test(XLen::X32, "rv32ui-p-srai");
    runner.test(XLen::X32, "rv32ui-p-srl");
    runner.test(XLen::X32, "rv32ui-p-srli");
    runner.test(XLen::X32, "rv32ui-p-sub");
    runner.test(XLen::X32, "rv32ui-p-sw");
    runner.test(XLen::X32, "rv32ui-p-xor");
    runner.test(XLen::X32, "rv32ui-p-xori");

    //mi-p-*
    runner.test(XLen::X64, "rv64mi-p-access");
    runner.test(XLen::X64, "rv64mi-p-breakpoint");
    runner.test(XLen::X64, "rv64mi-p-sbreak");
    runner.test(XLen::X64, "rv64mi-p-csr");
    runner.test(XLen::X64, "rv64mi-p-illegal");
    runner.test(XLen::X64, "rv64mi-p-ma_addr");
    runner.test(XLen::X64, "rv64mi-p-ma_fetch");
    runner.test(XLen::X64, "rv64mi-p-mcsr");
    runner.test(XLen::X64, "rv64mi-p-scall");

    runner.test(XLen::X32, "rv32mi-p-breakpoint");
    runner.test(XLen::X32, "rv32mi-p-sbreak");
    runner.test(XLen::X32, "rv32mi-p-csr");
    runner.test(XLen::X32, "rv32mi-p-illegal");
    runner.test(XLen::X32, "rv32mi-p-ma_addr");
    runner.test(XLen::X32, "rv32mi-p-ma_fetch");
    runner.test(XLen::X32, "rv32mi-p-mcsr");
    runner.test(XLen::X32, "rv32mi-p-scall");
    runner.test(XLen::X32, "rv32mi-p-shamt");

    //si-p-*
    runner.test(XLen::X64, "rv64si-p-csr");
    runner.test(XLen::X64, "rv64si-p-dirty");
    runner.test(XLen::X64, "rv64si-p-ma_fetch");
    runner.test(XLen::X64, "rv64si-p-sbreak");
    runner.test(XLen::X64, "rv64si-p-scall");
    runner.test(XLen::X64, "rv64si-p-wfi");

    runner.test(XLen::X32, "rv32si-p-csr");
    runner.test(XLen::X32, "rv32si-p-dirty");
    runner.test(XLen::X32, "rv32si-p-ma_fetch");
    runner.test(XLen::X32, "rv32si-p-sbreak");
    runner.test(XLen::X32, "rv32si-p-scall");
    runner.test(XLen::X32, "rv32si-p-wfi");

    //um-p-*
    runner.test(XLen::X64, "rv64um-p-mul");
    runner.test(XLen::X64, "rv64um-p-mulh");
    runner.test(XLen::X64, "rv64um-p-mulhsu");
    runner.test(XLen::X64, "rv64um-p-mulhu");
    runner.test(XLen::X64, "rv64um-p-mulw");
    runner.test(XLen::X64, "rv64um-p-div");
    runner.test(XLen::X64, "rv64um-p-divu");
    runner.test(XLen::X64, "rv64um-p-divw");
    runner.test(XLen::X64, "rv64um-p-divuw");
    runner.test(XLen::X64, "rv64um-p-rem");
    runner.test(XLen::X64, "rv64um-p-remu");
    runner.test(XLen::X64, "rv64um-p-remw");
    runner.test(XLen::X64, "rv64um-p-remuw");

    runner.test(XLen::X32, "rv32um-p-mul");
    runner.test(XLen::X32, "rv32um-p-mulh");
    runner.test(XLen::X32, "rv32um-p-mulhsu");
    runner.test(XLen::X32, "rv32um-p-mulhu");
    runner.test(XLen::X32, "rv32um-p-div");
    runner.test(XLen::X32, "rv32um-p-divu");
    runner.test(XLen::X32, "rv32um-p-rem");
    runner.test(XLen::X32, "rv32um-p-remu");

    //uf-p-*
    runner.test(XLen::X64, "rv64uf-p-fadd");
    runner.test(XLen::X64, "rv64uf-p-fclass");
    runner.test(XLen::X64, "rv64uf-p-fcmp");
    runner.test(XLen::X64, "rv64uf-p-fcvt");
    runner.test(XLen::X64, "rv64uf-p-fcvt_w");
    runner.test(XLen::X64, "rv64uf-p-fdiv");
    runner.test(XLen::X64, "rv64uf-p-fmadd");
    runner.test(XLen::X64, "rv64uf-p-fmin");
    runner.test(XLen::X64, "rv64uf-p-ldst");
    runner.test(XLen::X64, "rv64uf-p-move");
    runner.test(XLen::X64, "rv64uf-p-recoding");

    runner.test(XLen::X32, "rv32uf-p-fadd");
    runner.test(XLen::X32, "rv32uf-p-fclass");
    runner.test(XLen::X32, "rv32uf-p-fcmp");
    runner.test(XLen::X32, "rv32uf-p-fcvt");
    runner.test(XLen::X32, "rv32uf-p-fcvt_w");
    runner.test(XLen::X32, "rv32uf-p-fdiv");
    runner.test(XLen::X32, "rv32uf-p-fmadd");
    runner.test(XLen::X32, "rv32uf-p-fmin");
    runner.test(XLen::X32, "rv32uf-p-ldst");
    runner.test(XLen::X32, "rv32uf-p-move");
    runner.test(XLen::X32, "rv32uf-p-recoding");

    //ud-p-*
    runner.test(XLen::X64, "rv64ud-p-fadd");
    runner.test(XLen::X64, "rv64ud-p-fclass");
    runner.test(XLen::X64, "rv64ud-p-fcmp");
    runner.test(XLen::X64, "rv64ud-p-fcvt");
    runner.test(XLen::X64, "rv64ud-p-fcvt_w");
    runner.test(XLen::X64, "rv64ud-p-fdiv");
    runner.test(XLen::X64, "rv64ud-p-fmadd");
    runner.test(XLen::X64, "rv64ud-p-fmin");
    runner.test(XLen::X64, "rv64ud-p-ldst");
    runner.test(XLen::X64, "rv64ud-p-move");
    runner.test(XLen::X64, "rv64ud-p-recoding");
    runner.test(XLen::X64, "rv64ud-p-structural");

    runner.test(XLen::X32, "rv32ud-p-fadd");
    runner.test(XLen::X32, "rv32ud-p-fclass");
    runner.test(XLen::X32, "rv32ud-p-fcmp");
    runner.test(XLen::X32, "rv32ud-p-fcvt");
    runner.test(XLen::X32, "rv32ud-p-fcvt_w");
    runner.test(XLen::X32, "rv32ud-p-fdiv");
    runner.test(XLen::X32, "rv32ud-p-fmadd");
    runner.test(XLen::X32, "rv32ud-p-fmin");
    runner.test(XLen::X32, "rv32ud-p-ldst");
    runner.test(XLen::X32, "rv32ud-p-recoding");

    //uc-p-*
    runner.test(XLen::X64, "rv64uc-p-rvc");

    runner.test(XLen::X32, "rv32uc-p-rvc");

    //ua-p-*
    runner.test(XLen::X64, "rv64ua-p-amoadd_d");
    runner.test(XLen::X64, "rv64ua-p-amoadd_w");
    runner.test(XLen::X64, "rv64ua-p-amoand_d");
    runner.test(XLen::X64, "rv64ua-p-amoand_w");
    runner.test(XLen::X64, "rv64ua-p-amomax_d");
    runner.test(XLen::X64, "rv64ua-p-amomax_w");
    runner.test(XLen::X64, "rv64ua-p-amomaxu_d");
    runner.test(XLen::X64, "rv64ua-p-amomaxu_w");
    runner.test(XLen::X64, "rv64ua-p-amomin_d");
    runner.test(XLen::X64, "rv64ua-p-amomin_w");
    runner.test(XLen::X64, "rv64ua-p-amominu_d");
    runner.test(XLen::X64, "rv64ua-p-amominu_w");
    runner.test(XLen::X64, "rv64ua-p-amoor_d");
    runner.test(XLen::X64, "rv64ua-p-amoor_w");
    runner.test(XLen::X64, "rv64ua-p-amoswap_d");
    runner.test(XLen::X64, "rv64ua-p-amoswap_w");
    runner.test(XLen::X64, "rv64ua-p-amoxor_d");
    runner.test(XLen::X64, "rv64ua-p-amoxor_w");
    runner.test(XLen::X64, "rv64ua-p-lrsc");

    runner.test(XLen::X32, "rv32ua-p-amoadd_w");
    runner.test(XLen::X32, "rv32ua-p-amoand_w");
    runner.test(XLen::X32, "rv32ua-p-amomax_w");
    runner.test(XLen::X32, "rv32ua-p-amomaxu_w");
    runner.test(XLen::X32, "rv32ua-p-amomin_w");
    runner.test(XLen::X32, "rv32ua-p-amominu_w");
    runner.test(XLen::X32, "rv32ua-p-amoor_w");
    runner.test(XLen::X32, "rv32ua-p-amoswap_w");
    runner.test(XLen::X32, "rv32ua-p-amoxor_w");
    runner.test(XLen::X32, "rv32ua-p-lrsc");


    //*-pm-*
    runner.test_mp(XLen::X64, "rv64ui-pm-add", 2);
    runner.test_mp(XLen::X64, "rv64ui-pm-addi",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-addiw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-addw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-and",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-andi",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-auipc",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-beq",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-bge",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-bgeu",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-blt",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-bltu",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-bne",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-fence_i",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-jal",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-jalr",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-lb",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-lbu",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-ld",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-lh",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-lhu",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-lui",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-lw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-lwu",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-or",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-ori",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sb",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sd",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sh",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-simple",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sll",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-slli",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-slliw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sllw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-slt",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-slti",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sltiu",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sltu",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sra",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-srai",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sraiw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sraw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-srl",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-srli",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-srliw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-srlw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-sub",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-subw",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-xor",2);
    runner.test_mp(XLen::X64, "rv64ui-pm-xori",2);
    runner.test_mp(XLen::X32, "rv32ui-pm-add" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-addi" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-and" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-andi" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-auipc" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-beq" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-bge" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-bgeu" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-blt" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-bltu" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-bne" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-fence_i" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-jal" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-jalr" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-lb" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-lbu" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-lh" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-lhu" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-lui" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-lw" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-or" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-ori" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-sb" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-sh" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-simple" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-sll" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-slli" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-slt" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-slti" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-sltiu" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-sltu" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-sra" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-srai" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-srl" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-srli" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-sub" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-sw" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-xor" ,2);
    runner.test_mp(XLen::X32, "rv32ui-pm-xori" ,2);

    runner.test_mp(XLen::X64, "rv64ua-pm-lrsc" ,4);
    runner.test_mp(XLen::X32, "rv32ua-pm-lrsc" ,4);

    //*-v-*
    runner.test(XLen::X64, "rv64ui-v-add");
    runner.test(XLen::X32, "rv32ui-v-jalr");
    runner.test(XLen::X64, "rv64um-v-mul");
    runner.test(XLen::X32, "rv32um-v-div");
    runner.test(XLen::X64, "rv64uf-v-fmadd");
    runner.test(XLen::X32, "rv32uf-v-fdiv");
    runner.test(XLen::X64, "rv64ud-v-move");
    runner.test(XLen::X32, "rv32ud-v-fcvt_w");
    runner.test(XLen::X64, "rv64uc-v-rvc");
    runner.test(XLen::X32, "rv32uc-v-rvc");
    runner.test(XLen::X64, "rv64ua-v-amoadd_d");
    runner.test(XLen::X32, "rv32ua-v-amoadd_w");
    runner.test(XLen::X64, "rv64ua-v-lrsc");
    runner.test(XLen::X32, "rv32ua-v-lrsc");
}


