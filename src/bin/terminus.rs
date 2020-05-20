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
use terminus::devices::virtio_blk::{VirtIOBlk, VirtIOBlkConfig};
use terminus::devices::virtio_net::{VirtIONetDevice, VirtIONet};
#[cfg(feature = "sdl")]
use terminus_spaceport::devices::SDL;
use terminus::devices::display::{Fb, SimpleFb, DummyKb, DummyMouse};
use std::ops::Deref;
use std::time::Duration;


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

    let configs = vec![ProcessorCfg {
        xlen,
        enable_dirty: true,
        extensions,
        freq: 1000000000,
    }; core_num];
    let mut sys = System::new("sys", elf, configs, !virtio_input_en, 10000000, 32);
    let main_memory = GHEAP.alloc(memory_size, 1).expect("main_memory alloc fail!");
    sys.register_memory("main_memory", 0x80000000, &main_memory).unwrap();
    sys.register_device("clint", 0x02000000, 0x000c0000, Clint::new(sys.timer())).unwrap();
    sys.register_device("plic", 0x0c000000, 0x4000000, Plic::new(sys.intc())).unwrap();
    #[cfg(feature = "sdl")]
        let (sdl, fb, kb, mouse) = {
        let sdl = SDL::new("terminus", 800, 600, || { EXIT_CTRL.exit("sdl exit!").unwrap() }).expect("sdl open fail!");
        let fb = Rc::new(Fb::new(800, 600));
        let kb = DummyKb {};
        let mouse = DummyMouse {};
        sys.register_device("simple_fb", 0x30000000, fb.size() as u64, SimpleFb::new(&fb)).unwrap();
        (sdl, fb, kb, mouse)
    };

    //virtios
    let virtio_mem = Region::remap(0x80000000, &main_memory);
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
    //use virtio console
    sys.make_boot_rom(0x20000000, -1i64 as u64, boot_args).unwrap();
    sys.load_elf().unwrap();
    sys.reset(vec![-1i64 as u64; core_num]).unwrap();
    let mut real_timer = std::time::Instant::now();
    let interval = Duration::new(0, 1_000_000_000u32 / 60);
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            eprintln!("{}", msg);
            break;
        }
        for p in sys.processors() {
            p.step(5000);
        }
        if virtio_input_en {
            virtio_console_device.console_read();
        }
        if let Some(ref net_d) = virtio_net_device {
            net_d.net_read();
        }
        if real_timer.elapsed() >= interval {
            #[cfg(feature = "sdl")]
                sdl.refresh(fb.deref(), &kb, &mouse).unwrap();
            real_timer = std::time::Instant::now();
        }
        sys.timer().tick(50)
    }
    term_exit();
}