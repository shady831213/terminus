use terminus::processor::ProcessorCfg;
use terminus::system::System;
use std::path::Path;
use terminus::devices::clint::Clint;
use terminus_spaceport::devices::term_exit;
use terminus_spaceport::EXIT_CTRL;
use terminus_spaceport::memory::region::GHEAP;
use terminus::global::XLen;

fn main() {
    let num_cores = 1;
    let configs = vec![ProcessorCfg {
        xlen: XLen::X64,
        enable_dirty: true,
        extensions: vec!['m', 'f', 'd', 's', 'u', 'c', 'a'].into_boxed_slice(),
        freq: 1000000000,
    }; num_cores];
    let mut sys = System::new("sys", Path::new("examples/linux/image/br-5-4").to_str().expect("image not found!"), 10000000, 32);
    sys.register_htif(true);
    for cfg in configs {
        sys.new_processor(cfg)
    }
    sys.register_memory("main_memory", 0x80000000, &GHEAP.alloc(0x80000000, 1).expect("main_memory alloc fail!")).unwrap();
    sys.register_device("clint", 0x02000000, 0x000c0000, Clint::new(sys.timer())).unwrap();
    sys.make_boot_rom(0x20000000, -1i64 as u64, vec![]).unwrap();
    sys.load_elf().unwrap();
    sys.reset(vec![-1i64 as u64; num_cores]).unwrap();
    // let interval: u64 = 100;
    // let mut interval_cnt: u64 = 0;
    'outer:loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            eprintln!("{}", msg);
            break;
        }
        for p in sys.processors() {
            p.step(5000);
            // eprintln!("{}", p.state().trace());
            // if *p.state().pc() == 0x0000000080000044{
            //     p.step(1);
            //     break 'outer;
            // }
        }
        sys.timer().tick(50)
        // interval_cnt += 1;
        // if interval_cnt % interval == interval - 1 {
        //     sys.timer().tick(1)
        // }
    }
    eprintln!("{}", sys.processor(0).unwrap().state().to_string());
    term_exit();
}
