use terminus_spaceport::memory::MemInfo;
use terminus_spaceport::space::Space;
use terminus_spaceport::space;
use terminus_spaceport::memory::region::{Region, IOAccess, BytesAccess, GHEAP};
use std::fmt;
use crate::devices::htif::HTIF;
use crate::devices::bus::Bus;
use std::fmt::{Display, Formatter};
use crate::processor::{ProcessorCfg, Processor};
use std::cmp::min;
use crate::devices::clint::Timer;
use std::ops::Deref;
use std::rc::Rc;
use terminus_global::XLen;
use crate::devices::plic::Intc;

pub mod fdt;

use fdt::{FdtNode, FdtProp};

pub mod elf;

use elf::ElfLoader;
use terminus_spaceport::virtio::{MMIODevice, VirtIOInfo};

const VIRTIO_BASE: u64 = 0x40000000;
const VIRTIO_SIZE: u64 = 0x1000;

#[derive(Debug)]
pub enum Error {
    SpaceErr(space::Error),
    ElfErr(String),
    FdtErr(String),
    ResetErr(String),
}

impl From<space::Error> for Error {
    fn from(v: space::Error) -> Error {
        Error::SpaceErr(v)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct System {
    name: String,
    bus: Rc<Bus>,
    timer: Rc<Timer>,
    intc: Rc<Intc>,
    elf: ElfLoader,
    processors: Vec<Processor>,
    virtio_infos: Vec<VirtIOInfo>,
}

impl System {
    pub fn new(name: &str, elf_file: &str, processor_cfgs: Vec<ProcessorCfg>, timer_freq: usize, max_int_src: usize) -> System {
        let bus = Rc::new(Bus::new());
        let elf = ElfLoader::new(elf_file).expect(&format!("Invalid Elf {}", elf_file));
        let mut sys = System {
            name: name.to_string(),
            bus,
            timer: Rc::new(Timer::new(timer_freq)),
            intc: Rc::new(Intc::new(max_int_src)),
            elf,
            processors: vec![],
            virtio_infos: vec![],
        };
        sys.try_register_htif();
        for cfg in processor_cfgs {
            sys.new_processor(cfg)
        }
        sys
    }

    fn new_processor(&mut self, config: ProcessorCfg) {
        let p = Processor::new(self.processors.len(), config, &self.bus, self.timer.alloc_irq(), self.intc.alloc_irq());
        self.processors.push(p)
    }

    fn register_region(&self, name: &str, base: u64, region: &Rc<Region>) -> Result<()> {
        self.bus.space_mut().add_region(name, &Region::remap(base, &region))?;
        Ok(())
    }

    fn try_register_htif(&self) {
        if let Some((base, tohost, fromhost)) = self.elf.htif_section().expect("Invalid ELF!") {
            self.register_region("htif", base, &Region::io(0, 0x1000, Box::new(HTIF::new(tohost, fromhost)))).unwrap();
        }
    }

    pub fn processor(&mut self, hartid: usize) -> Option<&mut Processor> {
        if hartid >= self.processors.len() {
            None
        } else {
            Some(&mut self.processors[hartid])
        }
    }

    pub fn processors(&mut self) -> &mut Vec<Processor> {
        &mut self.processors
    }

    pub fn bus(&self) -> &Rc<Bus> {
        &self.bus
    }

    pub fn timer(&self) -> &Rc<Timer> {
        &self.timer
    }

    pub fn intc(&self) -> &Rc<Intc> {
        &self.intc
    }

    pub fn register_device<D: IOAccess + 'static>(&self, name: &str, base: u64, size: u64, device: D) -> Result<()> {
        self.register_region(name, base, &Region::io(0, size, Box::new(device)))
    }

    pub fn register_virtio<D: IOAccess + MMIODevice + 'static>(&mut self, name: &str, device: D) -> Result<()> {
        let id = self.virtio_infos.len() as u64;
        let base = VIRTIO_BASE + id * VIRTIO_SIZE;
        let irq_id = device.device().irq_id() as u32;
        self.register_device(name, base, VIRTIO_SIZE, device)?;
        self.virtio_infos.push(VirtIOInfo {
            base,
            size: VIRTIO_SIZE,
            irq_id,
            ty: "mmio".to_string(),
        });
        Ok(())
    }


    pub fn register_memory(&self, name: &str, base: u64, mem: &Rc<Region>) -> Result<()> {
        match self.register_region(name, base, &mem) {
            Ok(_) => { Ok(()) }
            Err(e) => {
                if let Error::SpaceErr(space::Error::Overlap(n, msg)) = e {
                    if n == "htif".to_string() {
                        let htif_region = self.bus.space().get_region(&n).unwrap();
                        let range0 = if base < htif_region.info.base {
                            Some(MemInfo { base: base, size: htif_region.info.base - base })
                        } else {
                            None
                        };
                        let range1 = if base + mem.info.size > htif_region.info.base + htif_region.info.size {
                            Some(MemInfo { base: htif_region.info.base + htif_region.info.size, size: base + mem.info.size - (htif_region.info.base + htif_region.info.size) })
                        } else {
                            None
                        };
                        range0.iter().for_each(|info| {
                            self.bus.space_mut().add_region(name, &Region::remap_partial(info.base, mem, 0, info.size)).unwrap();
                        });
                        range1.iter().for_each(|info| {
                            self.bus.space_mut().add_region(&format!("{}_1", name), &Region::remap_partial(info.base, mem, info.base - base, info.size)).unwrap();
                        });
                        Ok(())
                    } else {
                        Err(Error::from(space::Error::Overlap(n, msg)))
                    }
                } else {
                    Err(e)
                }
            }
        }
    }


    pub fn load_elf(&self) -> Result<()> {
        match self.elf.load(|addr, data| {
            fn load(space: &Space, addr: u64, data: &[u8]) -> std::result::Result<(), String> {
                if data.is_empty() {
                    Ok(())
                } else {
                    let region = space.get_region_by_addr(&addr).expect("not enough memory!");
                    let len = min((region.info.base + region.info.size - addr) as usize, data.len());
                    let (head, tails) = data.split_at(len);
                    BytesAccess::write(region.deref(), &addr, head).unwrap();
                    load(space, region.info.base + region.info.size, tails)
                }
            };
            load(self.bus.space().deref(), addr, data)
        }) {
            Ok(_) => Ok(()),
            Err(msg) => Err(Error::ElfErr(msg))
        }
    }

    fn compile_fdt(&self, boot_args: Vec<&str>) -> Result<Vec<u8>> {
        let mut root = FdtNode::new("");
        root.add_prop(FdtProp::u32_prop("#address-cells", vec![2]));
        root.add_prop(FdtProp::u32_prop("#size-cells", vec![2]));
        root.add_prop(FdtProp::str_prop("compatible", vec!["ucbbar,terminus-bare-dev"]));
        root.add_prop(FdtProp::str_prop("model", vec!["ucbbar,terminus-bare"]));

        let mut cpus = FdtNode::new("cpus");
        cpus.add_prop(FdtProp::u32_prop("#address-cells", vec![1]));
        cpus.add_prop(FdtProp::u32_prop("#size-cells", vec![0]));
        cpus.add_prop(FdtProp::u32_prop("timebase-frequency", vec![self.timer.freq() as u32]));

        for p in self.processors.iter() {
            let mut cpu = FdtNode::new_with_num("cpu", p.state().hartid() as u64);
            cpu.add_prop(FdtProp::str_prop("device_type", vec!["cpu"]));
            cpu.add_prop(FdtProp::u32_prop("reg", vec![p.state().hartid() as u32]));
            cpu.add_prop(FdtProp::str_prop("status", vec!["okay"]));
            cpu.add_prop(FdtProp::str_prop("compatible", vec!["riscv"]));
            cpu.add_prop(FdtProp::str_prop("riscv,isa", vec![&p.state().isa_string()]));
            cpu.add_prop(FdtProp::u32_prop("clock-frequency", vec![p.state().config().freq as u32]));
            match p.state().config().xlen {
                XLen::X64 => cpu.add_prop(FdtProp::str_prop("mmu-type", vec!["riscv,sv48"])),
                XLen::X32 => cpu.add_prop(FdtProp::str_prop("mmu-type", vec!["riscv,sv32"])),
            }
            let mut intc = FdtNode::new("interrupt-controller");
            intc.add_prop(FdtProp::u32_prop("#interrupt-cells", vec![1]));
            intc.add_prop(FdtProp::null_prop("interrupt-controller"));
            intc.add_prop(FdtProp::str_prop("compatible", vec!["riscv,cpu-intc"]));
            intc.add_prop(FdtProp::u32_prop("phandle", vec![(p.state().hartid() + 1) as u32]));
            cpu.add_node(intc);
            cpus.add_node(cpu)
        }
        root.add_node(cpus);

        if let Some(main_memory) = self.bus.space().get_region("main_memory") {
            let base = main_memory.info.base;
            let mut size = main_memory.info.size;
            //because of htif...
            if let Some(main_memory_1) = self.bus.space().get_region("main_memory_1") {
                let htif_region = self.bus.space().get_region("htif").unwrap();
                size += main_memory_1.info.size + htif_region.info.size
            }
            let mut memory = FdtNode::new_with_num("memory", base);
            memory.add_prop(FdtProp::str_prop("device_type", vec!["memory"]));
            memory.add_prop(FdtProp::u64_prop("reg", vec![base, size]));
            root.add_node(memory);
        } else {
            return Err(Error::FdtErr("\"main_memory\" is not in memory space!".to_string()));
        }

        let mut soc = FdtNode::new("soc");
        soc.add_prop(FdtProp::u32_prop("#address-cells", vec![2]));
        soc.add_prop(FdtProp::u32_prop("#size-cells", vec![2]));
        soc.add_prop(FdtProp::str_prop("compatible", vec!["ucbbar,terminus-bare-soc", "simple-bus"]));
        soc.add_prop(FdtProp::null_prop("ranges"));

        if let Some(clint_region) = self.bus.space().get_region("clint") {
            let mut clint = FdtNode::new_with_num("clint", clint_region.info.base);
            clint.add_prop(FdtProp::str_prop("compatible", vec!["riscv,clint0"]));
            let mut interrupts_extended = vec![];
            for p in self.processors.iter() {
                interrupts_extended.push((p.state().hartid() + 1) as u32);
                interrupts_extended.push(3 as u32);
                interrupts_extended.push((p.state().hartid() + 1) as u32);
                interrupts_extended.push(7 as u32);
            }
            clint.add_prop(FdtProp::u32_prop("interrupts-extended", interrupts_extended));
            clint.add_prop(FdtProp::u64_prop("reg", vec![clint_region.info.base, clint_region.info.size]));
            soc.add_node(clint);
        } else {
            return Err(Error::FdtErr("\"clint\" is not in memory space!".to_string()));
        }

        let plic_phandle = self.processors.len() as u32 + 1;
        if let Some(plic_region) = self.bus.space().get_region("plic") {
            let num_ints = self.intc.num_src() as u32 - 1;
            if num_ints != 0 {
                let mut plic = FdtNode::new_with_num("plic", plic_region.info.base);
                plic.add_prop(FdtProp::u32_prop("#interrupt-cells", vec![1]));
                plic.add_prop(FdtProp::null_prop("interrupt-controller"));
                plic.add_prop(FdtProp::u32_prop("riscv,ndev", vec![num_ints]));
                plic.add_prop(FdtProp::u32_prop("riscv,max-priority", vec![0x7]));
                plic.add_prop(FdtProp::str_prop("compatible", vec!["riscv,plic0"]));
                let mut interrupts_extended = vec![];
                for p in self.processors.iter() {
                    interrupts_extended.push((p.state().hartid() + 1) as u32);
                    interrupts_extended.push(9 as u32);
                    interrupts_extended.push((p.state().hartid() + 1) as u32);
                    interrupts_extended.push(11 as u32);
                }
                plic.add_prop(FdtProp::u32_prop("interrupts-extended", interrupts_extended));
                plic.add_prop(FdtProp::u64_prop("reg", vec![plic_region.info.base, plic_region.info.size]));
                plic.add_prop(FdtProp::u32_prop("phandle", vec![plic_phandle]));
                soc.add_node(plic);
            }
        }

        if !self.virtio_infos.is_empty() {
            assert!(self.bus.space().get_region("plic").is_some());
            for info in self.virtio_infos.iter() {
                let mut virtio = FdtNode::new_with_num("virtio", info.base);
                virtio.add_prop(FdtProp::str_prop("compatible", vec![&format!("virtio,{}", info.ty)]));
                virtio.add_prop(FdtProp::u64_prop("reg", vec![info.base, info.size]));
                virtio.add_prop(FdtProp::u32_prop("interrupts-extended", vec![plic_phandle, info.irq_id]));
                soc.add_node(virtio)
            }
        }

        let mut htif = FdtNode::new("htif");
        htif.add_prop(FdtProp::str_prop("compatible", vec!["ucb,htif0"]));
        soc.add_node(htif);

        root.add_node(soc);

        let mut chosen = FdtNode::new("chosen");
        if boot_args.is_empty() {
            chosen.add_prop(FdtProp::str_prop("bootargs", vec!["console=hvc0 earlycon=sbi"]));
        } else {
            chosen.add_prop(FdtProp::str_prop("bootargs", boot_args));
        }
        root.add_node(chosen);
        // eprintln!("{}", root.to_string());
        Ok(fdt::compile(&root))
    }

    pub fn make_boot_rom(&mut self, base: u64, entry: u64, boot_args: Vec<&str>) -> Result<()> {
        let start_address = if entry == -1i64 as u64 {
            self.elf.entry_point().unwrap()
        } else {
            entry
        };
        let mut dtb = self.compile_fdt(boot_args)?;
        let mut reset_vec: Vec<u32> = vec![
            0x297,                                                            //auipc t0, 0x0
            0,                                                                //placeholder[addi   a1, t0, &dtb]
            0xf1402573,                                                       //csrr   a0, mhartid
            match self.processor(0).unwrap().state().config().xlen {
                XLen::X64 => 0x0182b283,                                      // ld     t0,24(t0)
                XLen::X32 => 0x0182a283,                                      //lw     t0,24(t0)
            },
            0x28067,                                                          // jr     t0
            0,
            (start_address as u32) & (-1i32 as u32),
            (start_address >> 32) as u32
        ];
        reset_vec[1] = 0x28593 + ((reset_vec.len() as u32) * 4 << 20);       //addi   a1, t0, &dtb
        let mut rom: Vec<u8> = vec![];
        for i in reset_vec {
            rom.append(&mut i.to_le_bytes().to_vec());
        }
        rom.append(&mut dtb);
        let rom_mem = GHEAP.alloc(rom.len() as u64, 1).expect("boot rom alloc fail!");
        BytesAccess::write(rom_mem.deref(), &rom_mem.info.base, &rom).unwrap();
        self.register_memory("boot_rom", base, &rom_mem)?;
        Ok(())
    }

    pub fn reset(&mut self, reset_vecs: Vec<u64>) -> Result<()> {
        if reset_vecs.len() != self.processors.len() {
            return Err(Error::ResetErr(format!("reset_vecs size {} is not match with processor num {}!", reset_vecs.len(), self.processors.len())));
        }
        self.timer.reset();
        let boot_rom = self.bus.space().get_region("boot_rom");
        let entry_point = self.elf.entry_point().unwrap();
        for (i, p) in self.processors().iter_mut().enumerate() {
            if let Err(msg) = if reset_vecs[i] == -1i64 as u64 {
                if let Some(ref boot_rom) = boot_rom {
                    p.reset(boot_rom.info.base)
                } else {
                    p.reset(entry_point)
                }
            } else {
                p.reset(reset_vecs[i])
            } {
                return Err(Error::ResetErr(msg));
            }
        }
        Ok(())
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Machine {}:", self.name)?;
        writeln!(f, "   {}", self.bus.space().to_string())
    }
}


