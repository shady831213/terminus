use terminus_spaceport::memory::region::{Region, BytesAccess, U8Access, U16Access, U32Access, U64Access, IOAccess};
use terminus_spaceport::memory::{MemInfo, region};
use terminus_spaceport::space::{Space, SPACE_TABLE};
use terminus_spaceport::space;
use terminus_spaceport::derive_io;
use std::sync::{Arc, Mutex};
use std::{fmt, io, thread};
use super::elf::ElfLoader;
use std::ops::Deref;
use super::devices::htif::HTIF;
use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use terminus_global::RegT;
use std::sync::mpsc::{Sender, Receiver, channel, SendError, RecvError, TryRecvError};
use crate::processor::{ProcessorCfg, Processor};
use std::thread::JoinHandle;

#[derive_io(U8, U16, U32, U64)]
pub struct Bus {
    space: Arc<Space>
}

impl Bus {
    pub fn new(space: &Arc<Space>) -> Bus {
        Bus { space: space.clone() }
    }
}

impl U8Access for Bus {
    fn write(&self, addr: u64, data: u8) -> region::Result<()> {
        U8Access::write(self.space.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u8> {
        U8Access::read(self.space.deref(), addr)
    }
}

impl U16Access for Bus {
    fn write(&self, addr: u64, data: u16) -> region::Result<()> {
        U16Access::write(self.space.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u16> {
        U16Access::read(self.space.deref(), addr)
    }
}

impl U32Access for Bus {
    fn write(&self, addr: u64, data: u32) -> region::Result<()> {
        U32Access::write(self.space.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u32> {
        U32Access::read(self.space.deref(), addr)
    }
}

impl U64Access for Bus {
    fn write(&self, addr: u64, data: u64) -> region::Result<()> {
        U64Access::write(self.space.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u64> {
        U64Access::read(self.space.deref(), addr)
    }
}


pub enum SimCmd {
    RunOne,
    RunAll,
    RunN(usize),
}

pub enum SimResp {
    RunOne(bool),
    RunAll(usize),
    RunN(usize),
}

pub struct SimCmdSink {
    cmd: Receiver<SimCmd>,
    resp: Sender<SimResp>,
}

impl SimCmdSink {
    pub fn cmd(&self) -> &Receiver<SimCmd> {
        &self.cmd
    }
    pub fn resp(&self) -> &Sender<SimResp> {
        &self.resp
    }
}

struct SimCmdSource {
    cmd: Sender<SimCmd>,
    resp: Receiver<SimResp>,
}

pub struct SimController {
    channels: Mutex<HashMap<RegT, SimCmdSource>>
}

#[derive(Debug)]
pub enum SimCtrlError {
    HartIdExisted,
    HartIdNotExisted,
    CmdSendError(SendError<SimCmd>),
    RespSendError(SendError<SimResp>),
    RecvError(RecvError),
    TryRecvError(TryRecvError),
}

impl From<SendError<SimCmd>> for SimCtrlError {
    fn from(e: SendError<SimCmd>) -> SimCtrlError {
        SimCtrlError::CmdSendError(e)
    }
}

impl From<SendError<SimResp>> for SimCtrlError {
    fn from(e: SendError<SimResp>) -> SimCtrlError {
        SimCtrlError::RespSendError(e)
    }
}

impl From<RecvError> for SimCtrlError {
    fn from(e: RecvError) -> SimCtrlError {
        SimCtrlError::RecvError(e)
    }
}

impl From<TryRecvError> for SimCtrlError {
    fn from(e: TryRecvError) -> SimCtrlError {
        SimCtrlError::TryRecvError(e)
    }
}

impl SimController {
    fn new() -> SimController {
        SimController {
            channels: Mutex::new(HashMap::new())
        }
    }

    pub fn register_ch(&self, hartid: RegT) -> Result<SimCmdSink, SimCtrlError> {
        let (cmd_sender, cmd_receiver) = channel();
        let (resp_sender, resp_receiver) = channel();
        if let Some(_) = self.channels.lock().unwrap().insert(hartid, SimCmdSource { cmd: cmd_sender, resp: resp_receiver }) {
            Err(SimCtrlError::HartIdExisted)
        } else {
            Ok(SimCmdSink { cmd: cmd_receiver, resp: resp_sender })
        }
    }

    pub fn send_cmd(&self, hartid: RegT, cmd: SimCmd) -> Result<SimResp, SimCtrlError> {
        if let Some(ch) = self.channels.lock().unwrap().get(&hartid) {
            ch.cmd.send(cmd)?;
            Ok(ch.resp.recv()?)
        } else {
            Err(SimCtrlError::HartIdNotExisted)
        }
    }
}

pub struct System {
    name: String,
    mem_space: Arc<Space>,
    bus: Arc<Bus>,
    elf: ElfLoader,
    sim_controller: SimController,
}

impl System {
    pub fn new(name: &str, elf_file: &str) -> System {
        let space = SPACE_TABLE.get_space(name);
        let bus = Arc::new(Bus::new(&space));
        let elf = ElfLoader::new(elf_file).expect(&format!("Invalid Elf {}", elf_file));
        let sys = System {
            name: name.to_string(),
            mem_space: space,
            bus,
            elf,
            sim_controller: SimController::new(),
        };
        sys.try_register_htif();
        sys
    }

    pub fn new_processor<F: Fn(&Processor) + std::marker::Send + 'static>(&self, name: &str, config: ProcessorCfg, f: F) -> io::Result<JoinHandle<()>> {
        thread::Builder::new().name(name.to_string()).spawn({
            let bus = self.bus().clone();
            let hartid = config.hartid;
            let sink = self.sim_controller().register_ch(hartid).unwrap();
            move || {
                let p = Processor::new(config, &bus, sink);
                f(&p);
            }
        })
    }


    fn register_region(&self, name: &str, base: u64, region: &Arc<Region>) -> Result<Arc<Region>, space::Error> {
        self.mem_space.add_region(name, &Region::remap(base, &region))
    }

    fn try_register_htif(&self) {
        if let Some(s) = self.elf.htif_section().expect("Invalid ELF!") {
            self.register_region("htif", s.address(), &Region::io(0, 0x1000, Box::new(HTIF::new()))).unwrap();
        }
    }

    pub fn bus(&self) -> &Arc<Bus> {
        &self.bus
    }

    pub fn sim_controller(&self) -> &SimController {
        &self.sim_controller
    }

    pub fn mem_space(&self) -> &Arc<Space> {
        &self.mem_space
    }

    pub fn register_memory(&self, name: &str, base: u64, mem: &Arc<Region>) {
        match self.register_region(name, base, &mem) {
            Ok(_) => {}
            Err(space::Error::Overlap(n, msg)) => {
                if n == "htif".to_string() {
                    let htif_region = self.mem_space.get_region(&n).unwrap();
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
                        self.register_region(&format!("{}_0", name), info.base, &Region::remap_partial(0, mem, 0, info.size)).unwrap();
                    });
                    range1.iter().for_each(|info| {
                        self.register_region(&format!("{}_1", name), info.base, &Region::remap_partial(0, mem, info.base - base, info.size)).unwrap();
                    });
                } else {
                    panic!(msg)
                }
            }
            Err(space::Error::Renamed(_, msg)) => panic!(msg)
        }
    }

    pub fn entry_point(&self) -> Result<u64, String> {
        self.elf.entry_point()
    }

    pub fn load_elf(&self) {
        self.elf.load(|addr, data| {
            let region = self.mem_space.get_region_by_addr(addr).unwrap();
            if addr + data.len() as u64 > region.info.base + region.info.size {
                Err(format!("not enough memory!"))
            } else {
                if let Err(e) = BytesAccess::write(region.deref(), addr, data) {
                    Err(format!("{:?}", e))
                } else {
                    Ok(())
                }
            }
        }).expect(&format!("{} load elf fail!", self.name));
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Machine {}:", self.name)?;
        writeln!(f, "   {}", self.mem_space.to_string())
    }
}


