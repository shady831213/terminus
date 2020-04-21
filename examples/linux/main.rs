use terminus::processor::ProcessorCfg;
use terminus::system::System;
use std::path::Path;
use terminus::devices::clint::Clint;
use terminus_spaceport::devices::term_exit;
use terminus_spaceport::EXIT_CTRL;
use terminus_spaceport::memory::region::GHEAP;
use terminus_global::XLen;

fn main() {
    let num_cores = 1;
    let configs = vec![ProcessorCfg {
        xlen: XLen::X64,
        enable_dirty: true,
        extensions: vec!['m', 'f', 'd', 's', 'u', 'c', 'a'].into_boxed_slice(),
        freq: 1000000000,
    }; num_cores];
    let sys = System::new("sys", Path::new("examples/linux/image/br-base-bin-nodisk").to_str().expect("image not found!"), configs, 10000000);
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x10000000, 1).expect("main_memory alloc fail!")).unwrap();
    sys.register_device("clint", 0x02000000, 0x000c0000, Clint::new(sys.timer())).unwrap();
    sys.make_boot_rom(0x20000000, -1i64 as u64).unwrap();
    sys.load_elf().unwrap();
    sys.reset(vec![-1i64 as u64; num_cores]).unwrap();

    let interval: u64 = 100;
    let mut interval_cnt: u64 = 0;
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            eprintln!("{}", msg);
            break;
        }
        for p in sys.processors() {
            p.step(1);
            // eprintln!("{}", p.state().trace())
        }
        interval_cnt += 1;
        if interval_cnt % interval == interval - 1 {
            sys.timer().tick(interval)
        }
    }
    term_exit();
}
