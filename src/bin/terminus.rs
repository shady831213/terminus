extern crate clap;

use clap::{Arg, App};
use std::str::FromStr;
use terminus_global::XLen;
use std::path::Path;
use terminus::processor::ProcessorCfg;
use terminus::system::System;
use terminus::devices::clint::Clint;
use terminus_spaceport::devices::term_exit;
use terminus_spaceport::EXIT_CTRL;
use terminus_spaceport::memory::region::{GHEAP, Region};
use terminus::devices::plic::Plic;
use std::rc::Rc;
use terminus::devices::virtio_console::{VirtIOConsoleDevice, VirtIOConsole};

fn main() {
    let matches = App::new("terminus")
        .version("0.1")
        .author("Yang Li <shady831213@126.com>")
        .arg(Arg::with_name("core_num")
            .short("p")
            .value_name("CORE_NUM")
            .takes_value(true)
            .help("set core num, must be decimal int")
            .default_value("1")
        )
        .arg(Arg::with_name("memory")
            .short("m")
            .value_name("MEMORY_SIZE")
            .takes_value(true)
            .help("set memory size, must be hex int")
            .validator(|raw| {
                if raw.starts_with("0x") {
                    Ok(())
                } else {
                    Err(String::from("-m expect a hex int, start with 0x"))
                }
            })
            .default_value("0x80000000"))
        .arg(Arg::with_name("xlen")
            .short("l")
            .takes_value(true)
            .help("set xlen, 64 or 32")
            .validator(|raw| {
                match raw.as_str() {
                    "32" | "64" => Ok(()),
                    _ => Err(String::from("-l 64 or 32"))
                }
            })
            .default_value("64")
        )
        .arg(Arg::with_name("extensions")
            .short("e")
            .takes_value(true)
            .require_delimiter(true)
            .validator(|raw| {
                match raw.split_whitespace().collect::<String>().as_str() {
                    "a" | "c" | "d" | "f" | "i" | "m" | "s" | "u" => Ok(()),
                    _ => return Err(String::from("only support 'a', 'c', 'd', 'f', 'i', 'm', 's', 'u'"))
                }
            })
            .default_value("a, c, d, f, i, m, s, u")
        )
        .arg(
            Arg::with_name("elf")
                .index(1)
                .required(true)
                .value_name("ELF_FILE")
                .validator(|path| {
                    if Path::new(path.as_str()).exists() {
                        Ok(())
                    } else {
                        Err(format!("{} not exists!", path))
                    }
                })
                .help("elf file path")
        )
        .get_matches();

    let core_num = usize::from_str(matches.value_of("core_num").unwrap_or_default()).expect("-p expect a decimal int");
    let memory_size = u64::from_str_radix(matches.value_of("memory").unwrap_or_default().trim_start_matches("0x"), 16).expect("-m expect a hex int");
    let xlen = match matches.value_of("xlen").unwrap_or_default() {
        "32" => XLen::X32,
        "64" => XLen::X64,
        _ => unreachable!()
    };
    let extensions = matches.values_of("extensions").unwrap_or_default().map(|s| -> char{ s.split_whitespace().collect::<String>().chars().last().unwrap() }).collect::<Vec<char>>().into_boxed_slice();
    let elf = Path::new(matches.value_of("elf").unwrap()).to_str().unwrap();

    let configs = vec![ProcessorCfg {
        xlen,
        enable_dirty: true,
        extensions,
        freq: 1000000000,
    }; core_num];
    let mut sys = System::new("sys", elf, configs, 10000000, 32);
    let main_memory = GHEAP.alloc(memory_size, 1).expect("main_memory alloc fail!");
    sys.register_memory("main_memory", 0x80000000, &main_memory).unwrap();
    sys.register_device("clint", 0x02000000, 0x000c0000, Clint::new(sys.timer())).unwrap();
    sys.register_device("plic", 0x0c000000, 0x4000000, Plic::new(sys.intc())).unwrap();
    //virtios
    let virtio_mem = Region::remap(0x80000000, &main_memory);
    let irq_num = sys.intc().num_src();
    let virtio_console_device = Rc::new(VirtIOConsoleDevice::new(&virtio_mem, sys.intc().alloc_src(irq_num)));
    sys.register_virtio("virtio_console", VirtIOConsole::new(&virtio_console_device)).unwrap();

    sys.make_boot_rom(0x20000000, -1i64 as u64).unwrap();
    sys.load_elf().unwrap();
    sys.reset(vec![-1i64 as u64; core_num]).unwrap();
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            eprintln!("{}", msg);
            break;
        }
        for p in sys.processors() {
            p.step(5000);
        }
        // virtio_console_device.console_resize();
        // virtio_console_device.console_read();
        sys.timer().tick(50)
    }
    term_exit();
}