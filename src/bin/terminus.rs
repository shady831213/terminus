extern crate clap;

use clap::{Arg, App};
use std::str::FromStr;
use terminus_global::XLen;
use std::path::Path;
use std::fs::OpenOptions;
use terminus::processor::ProcessorCfg;
use terminus::system::System;
use terminus::devices::clint::Clint;
use terminus_spaceport::devices::term_exit;
use terminus_spaceport::EXIT_CTRL;
use terminus_spaceport::memory::region::{GHEAP, Region};
use terminus::devices::plic::Plic;
use std::rc::Rc;
use terminus::devices::virtio_console::{VirtIOConsoleDevice, VirtIOConsole};
use terminus::devices::virtio_blk::{VirtIOBlk, VirtIOBlkConfig};
use terminus::devices::virtio_net::{VirtIONetDevice, VirtIONet};
#[cfg(feature = "sdl")]
use terminus_spaceport::devices::{PixelFormat, FrameBuffer};
#[cfg(feature = "sdl")]
use terminus::devices::display::{Fb, SimpleFb};
#[cfg(feature = "sdl")]
use std::ops::Deref;
#[cfg(feature = "sdl")]
use std::time::Duration;
#[cfg(feature = "sdl")]
use terminus_spaceport::devices::SDL;
#[cfg(feature = "sdl")]
use terminus::system::fdt::FdtProp;
#[cfg(feature = "sdl")]
use terminus::devices::virtio_input::{VirtIOKbDevice, VirtIOKb};
#[cfg(feature = "sdl")]
use terminus::devices::virtio_input::{VirtIOMouseDevice, VirtIOMouse};
use std::io::Write;


fn main() {
    const CORE_FREQ:usize = 100000000;
    const TIMER_FREQ:usize = 10000000;
    const TIMER_STEP:u64 = 50;
    const CORE_STEP_TH:usize = 500;

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
        .arg(
            Arg::with_name("boot_args")
                .long("boot_args")
                .value_name("BOOT_ARGS")
                .takes_value(true)
                .require_delimiter(true)
                .help("set boot args")
                .default_value("root=/dev/vda console=hvc1 earlycon=sbi")
        )
        .arg(
            Arg::with_name("image")
                .long("image")
                .value_name("IMAGE")
                .takes_value(true)
                .help("image file")
        )
        .arg(
            Arg::with_name("image_mode")
                .long("image_mode")
                .value_name("IMAGE_MODE")
                .takes_value(true)
                .validator(|mode| {
                    match mode.as_str() {
                        "ro" | "rw" | "snapshot" => Ok(()),
                        _ => Err("image file mode:[ro|rw|snapshot]".to_string())
                    }
                })
                .help("image file mode:[ro|rw|snapshot]")
                .default_value("snapshot")
        )
        .arg(
            Arg::with_name("net")
                .long("net")
                .value_name("net")
                .takes_value(true)
                .help("tap iface name.
    config in host:
        sudo ip link add br0 type bridge
        sudo ip tuntap add dev [tap iface] mode tap user $(whoami)
        sudo ip link set [tap iface] master br0
        sudo ip link set dev br0 up
        sudo ip link set dev [tap iface] up
        sudo ifconfig br0 192.168.3.1
        sudo sysctl -w net.ipv4.ip_forward=1
        sudo iptables --policy FORWARD ACCEPT
        sudo iptables -t nat -A POSTROUTING -o [eth] -j MASQUERADE
    config in guest:
        ifconfig eth0 192.168.3.2
        echo \"nameserver 8.8.8.8\" > /etc/resolv.conf
        route add -net 0.0.0.0 netmask 0.0.0.0 gw 192.168.3.1")
        )
        .arg(
            Arg::with_name("console_input")
                .long("console_input")
                .value_name("CONSOLE_INPUT")
                .takes_value(true)
                .help("config console input type:[htif|virtio]")
                .validator(|mode| {
                    match mode.as_str() {
                        "htif" | "virtio" => Ok(()),
                        _ => Err("config console input type:[htif|virtio]".to_string())
                    }
                })
                .default_value("virtio")
        )
        .arg(
            Arg::with_name("display")
                .long("display")
                .help("enable display device, need \"sdl\" feature")
        )
        .arg(
            Arg::with_name("width")
                .long("width")
                .value_name("WIDTH")
                .takes_value(true)
                .help("display width, need \"sdl\" feature")
                .default_value("800")
        )
        .arg(
            Arg::with_name("height")
                .long("height")
                .value_name("HEIGHT")
                .takes_value(true)
                .help("display height, need \"sdl\" feature")
                .default_value("600")
        )
        .arg(
            Arg::with_name("pixel_format")
                .long("pixel_format")
                .takes_value(true)
                .value_name("PIXEL_FORMAT")
                .help("display pixel format:[rgb566|rgb888], need \"sdl\" feature")
                .validator(|format| {
                    match format.as_str() {
                        "rgb565" | "rgb888" => Ok(()),
                        _ => Err("display pixel format:[rgb566|rgb888], need \"sdl\" feature".to_string())
                    }
                })
                .default_value("rgb565")
        )
        .arg(
            Arg::with_name("step")
                .long("step")
                .value_name("STEP")
                .takes_value(true)
                .help("CPU execution num in one step.")
                .default_value("500")
        )
        .arg(
            Arg::with_name("trace")
                .long("trace")
                .help("trace states all processors every tick interval(about 500 instructions), results is in terminus.trace")
        )
        .arg(
            Arg::with_name("trace_all")
                .long("trace_all")
                .help("trace states of all processors every instruction, results is in terminus.trace")
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
    let boot_args = matches.values_of("boot_args").unwrap_or_default().map(|s| { s }).collect::<Vec<_>>();
    let image = matches.value_of("image");
    let net = matches.value_of("net");
    let virtio_input_en = matches.value_of("console_input").unwrap_or_default() == "virtio";
    #[cfg(feature = "sdl")]
        let display_en = matches.is_present("display");
    #[cfg(feature = "sdl")]
        let display_width = u32::from_str(matches.value_of("width").unwrap_or_default()).expect("width expect a decimal");
    #[cfg(feature = "sdl")]
        let display_height = u32::from_str(matches.value_of("height").unwrap_or_default()).expect("height expect a decimal");
    #[cfg(feature = "sdl")]
        let pixel_format = match matches.value_of("pixel_format").unwrap_or_default() {
        "rgb565" => PixelFormat::RGB565,
        "rgb888" => PixelFormat::RGB888,
        _ => unreachable!()
    };
    let step = match usize::from_str(matches.value_of("step").unwrap_or_default()).expect("step expect a decimal") {
        0 => panic!("step can not be 0!"),
        s => s
    };
    let trace_all =  matches.is_present("trace_all");
    let mut trace_file = if matches.is_present("trace") || trace_all {
        Some(OpenOptions::new().create(true).write(true).open("terminus.trace").expect("Can not open terminus.trace!"))
    } else {
        None
    };


    let configs = vec![ProcessorCfg {
        xlen,
        enable_dirty: true,
        extensions,
        freq: CORE_FREQ,
    }; core_num];
    let mut sys = System::new("sys", elf, TIMER_FREQ, 32);
    sys.register_htif(!virtio_input_en);
    for cfg in configs {
        sys.new_processor(cfg)
    }
    let main_memory = GHEAP.alloc(memory_size, 1).expect("main_memory alloc fail!");
    sys.register_memory("main_memory", 0x80000000, &main_memory).unwrap();
    sys.register_device("clint", 0x02000000, 0x000c0000, Clint::new(sys.timer())).unwrap();
    sys.register_device("plic", 0x0c000000, 0x4000000, Plic::new(sys.intc())).unwrap();

    //virtios
    let virtio_mem = Region::remap(0x80000000, &main_memory);
    #[cfg(feature = "sdl")]
        let (sdl, fb, kb, mouse) = {
        if display_en {
            let sdl = SDL::new("terminus", display_width, display_height, pixel_format, || { EXIT_CTRL.exit("sdl exit!").unwrap() }).expect("sdl open fail!");
            let fb = Rc::new(Fb::new(display_width, display_height, pixel_format));
            let irq_num = sys.intc().num_src();
            let kb = Rc::new(VirtIOKbDevice::new(&virtio_mem, sys.intc().alloc_src(irq_num)));
            sys.register_virtio("virtio_keyboard", VirtIOKb::new(&kb)).unwrap();
            let irq_num = sys.intc().num_src();
            let mouse = Rc::new(VirtIOMouseDevice::new(&virtio_mem, sys.intc().alloc_src(irq_num)));
            sys.register_virtio("virtio_mouse", VirtIOMouse::new(&mouse)).unwrap();
            // let mouse = DummyMouse {};
            sys.register_device_with_fdt_props("simple_fb", 0x30000000, fb.size() as u64, SimpleFb::new(&fb), vec![
                FdtProp::u32_prop("width", vec![fb.width()]),
                FdtProp::u32_prop("height", vec![fb.height()]),
                FdtProp::u32_prop("stride", vec![fb.stride()]),
                match fb.pixel_format() {
                    PixelFormat::RGB565 => FdtProp::str_prop("format", vec!["r5g6b5"]),
                    PixelFormat::RGB888 => FdtProp::str_prop("format", vec!["a8r8g8b8"]),
                },
            ]).unwrap();
            (Some(sdl), Some(fb), Some(kb), Some(mouse))
        } else {
            (None, None, None, None)
        }
    };


    let irq_num = sys.intc().num_src();
    let virtio_console_device = Rc::new(VirtIOConsoleDevice::new(&virtio_mem, sys.intc().alloc_src(irq_num)));
    sys.register_virtio("virtio_console", VirtIOConsole::new(&virtio_console_device)).unwrap();

    if let Some(image_file) = image {
        let irq_num = sys.intc().num_src();
        let virtio_blk = VirtIOBlk::new(&virtio_mem, sys.intc().alloc_src(irq_num), 8,
                                        Path::new(image_file).to_str().expect("image not found!"), VirtIOBlkConfig::new(matches.value_of("image_mode").unwrap_or_default()));
        sys.register_virtio("virtio_blk", virtio_blk).unwrap();
    }
    let virtio_net_device = if let Some(tap) = net {
        let irq_num = sys.intc().num_src();
        let virtio_net_device = Rc::new(VirtIONetDevice::new(&virtio_mem, sys.intc().alloc_src(irq_num), tap, 0x0100_00000002));
        sys.register_virtio("virtio_net", VirtIONet::new(&virtio_net_device)).unwrap();
        Some(virtio_net_device)
    } else {
        None
    };

    sys.make_boot_rom(0x20000000, -1i64 as u64, boot_args).unwrap();
    sys.load_elf().unwrap();
    sys.reset(vec![-1i64 as u64; core_num]).unwrap();
    #[cfg(feature = "sdl")]
        let mut real_timer = if display_en { Some(std::time::Instant::now()) } else { None };
    #[cfg(feature = "sdl")]
        let interval = if display_en { Some(Duration::new(0, 1_000_000_000u32 / 30)) } else { None };
    let mut step_cnt:usize = 0;
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            eprintln!("{}", msg);
            break;
        }
        for p in sys.processors() {
            if let Some(ref mut f) = trace_file {
                p.step_with_debug(step, f, trace_all).unwrap()
            } else {
                p.step(step);
            }
        }
        step_cnt += step;
        if step_cnt >= CORE_STEP_TH {
            if virtio_input_en {
                virtio_console_device.console_read();
            }
            if let Some(ref net_d) = virtio_net_device {
                net_d.net_read();
            }
            #[cfg(feature = "sdl")]
                {
                    if let Some(ref display) = sdl {
                        let rt = real_timer.as_mut().unwrap();
                        if rt.elapsed() >= interval.unwrap() {
                            display.refresh(fb.as_ref().unwrap().deref(), kb.as_ref().unwrap().deref(), mouse.as_ref().unwrap().deref()).unwrap();
                            *rt += interval.unwrap()
                        }
                    }
                }
            sys.timer().tick(TIMER_STEP);
            step_cnt -= CORE_STEP_TH
        }
    }
    if let Some(ref mut f) = trace_file {
        for p in sys.processors() {
            f.write_all(p.state().to_string().as_bytes()).unwrap()
        }
    }
    term_exit();
}