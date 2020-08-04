use crate::system::System;
use crate::cosim::core::{CosimServer, ServerState};
use crate::processor::ProcessorCfg;
use crate::devices::clint::Clint;
use crate::devices::plic::Plic;

#[repr(u32)]
pub enum CosimCmdId {
    Reserved = 0,
    Reset,
    SysInit,
    AddProcessor,
    AddClint,
    AddPlic,
    InitDone,
}

#[derive(Debug, Copy, Clone)]
pub struct CosimCmdMeta {
    pub idx: u32,
    pub id: u32,
}

trait CosimServerCmd {
    fn id(&self) -> CosimCmdId;
    fn execute(&self, server: &mut CosimServer) -> Result<(), String>;
}

trait InitingCmd: CosimServerCmd {
    fn check_state(&self, server: &CosimServer) -> Result<(), String> {
        if server.state == ServerState::Initing {
            Ok(())
        } else {
            Err("init process has been done!".to_string())
        }
    }
}

trait NeedSysCmd: CosimServerCmd {
    fn get_sys<'a>(&self, server: &'a mut CosimServer) -> Result<&'a mut System, String> {
        if let Some(sys) = server.sys.as_mut() {
            Ok(sys)
        } else {
            Err("system not exist!".to_string())
        }
    }
}

trait RunningCmd: CosimServerCmd {
    fn check_state(&self, server: &CosimServer) -> Result<(), String> {
        if server.state == ServerState::Running {
            Ok(())
        } else {
            Err("init process not done!".to_string())
        }
    }
}

#[derive(Debug)]
pub struct CosimResp {
    pub meta: CosimCmdMeta,
    pub ty: CosimRespTy,
}

impl CosimResp {
    pub fn new(meta: CosimCmdMeta, ty: CosimRespTy) -> CosimResp {
        CosimResp {
            meta,
            ty,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum CosimRespTy {
    Ok,
    Err(String),
}

pub struct CosimCmd {
    meta: CosimCmdMeta,
    ty: CosimCmdTy,
}

impl CosimCmd {
    pub fn new(idx: u32, ty: CosimCmdTy) -> CosimCmd {
        CosimCmd {
            meta: CosimCmdMeta {
                idx,
                id: ty.id(),
            },
            ty,
        }
    }
    pub fn meta(&self) -> &CosimCmdMeta {
        &self.meta
    }
    pub fn ty(&self) -> &CosimCmdTy {
        &self.ty
    }
}

pub enum CosimCmdTy {
    Reset(ResetCmd),
    SysInit(SystemInitCmd),
    AddProcessor(AddProcessorCmd),
    AddClint(AddClintCmd),
    AddPlic(AddPlicCmd),
    InitDone(InitDoneCmd),
}

impl CosimCmdTy {
    fn id(&self) -> u32 {
        (match self {
            CosimCmdTy::Reset(cmd) => cmd.id(),
            CosimCmdTy::SysInit(cmd) => cmd.id(),
            CosimCmdTy::AddProcessor(cmd) => cmd.id(),
            CosimCmdTy::AddClint(cmd) => cmd.id(),
            CosimCmdTy::AddPlic(cmd) => cmd.id(),
            CosimCmdTy::InitDone(cmd) => cmd.id(),
        }) as u32
    }

    pub fn execute(&self, server: &mut CosimServer) -> CosimRespTy {
        let res = match self {
            CosimCmdTy::Reset(cmd) => cmd.execute(server),
            CosimCmdTy::SysInit(cmd) => cmd.execute(server),
            CosimCmdTy::AddProcessor(cmd) => cmd.execute(server),
            CosimCmdTy::AddClint(cmd) => cmd.execute(server),
            CosimCmdTy::AddPlic(cmd) => cmd.execute(server),
            CosimCmdTy::InitDone(cmd) => cmd.execute(server),
        };
        match res {
            Ok(_) => CosimRespTy::Ok,
            Err(e) => CosimRespTy::Err(e)
        }
    }
}

//commands
pub struct ResetCmd {}

impl CosimServerCmd for ResetCmd {
    fn id(&self) -> CosimCmdId {
        CosimCmdId::Reset
    }
    fn execute(&self, server: &mut CosimServer) -> Result<(), String> {
        server.reset();
        Ok(())
    }
}

impl CosimCmdTy {
    pub fn reset() -> CosimCmdTy {
        CosimCmdTy::Reset(ResetCmd{})
    }
}

pub struct SystemInitCmd {
    elf: String,
    htif_en:bool,
    max_int_src: usize,
}

impl CosimCmdTy {
    pub fn sys_init(elf: &str, htif_en:bool, max_int_src: usize) -> CosimCmdTy {
        CosimCmdTy::SysInit( SystemInitCmd {
            elf: elf.to_string(),
            htif_en,
            max_int_src,
        })
    }
}

impl InitingCmd for SystemInitCmd {}

impl CosimServerCmd for SystemInitCmd {
    fn id(&self) -> CosimCmdId {
        CosimCmdId::SysInit
    }
    fn execute(&self, server: &mut CosimServer) -> Result<(), String> {
        self.check_state(server)?;
        if server.sys.is_some() {
            return Err("system has been inited!".to_string());
        }
        server.sys = Some({
            let sys = System::new("cosim_sys", &self.elf, 10000000, self.max_int_src);
            if self.htif_en {
                sys.register_htif(true);
            }
            sys
        });
        Ok(())
    }
}

pub struct AddProcessorCmd {
    cfg: ProcessorCfg,
}

impl CosimCmdTy {
    pub fn add_processor(cfg:ProcessorCfg) -> CosimCmdTy {
        CosimCmdTy::AddProcessor( AddProcessorCmd {
            cfg,
        })
    }
}

impl InitingCmd for AddProcessorCmd {}

impl NeedSysCmd for AddProcessorCmd {}

impl CosimServerCmd for AddProcessorCmd {
    fn id(&self) -> CosimCmdId {
        CosimCmdId::AddProcessor
    }
    fn execute(&self, server: &mut CosimServer) -> Result<(), String> {
        self.check_state(server)?;
        let sys = self.get_sys(server)?;
        sys.new_processor(self.cfg.clone());
        Ok(())
    }
}

pub struct AddClintCmd {
    base: u64,
}

impl CosimCmdTy {
    pub fn add_clint(base:u64) -> CosimCmdTy {
        CosimCmdTy::AddClint( AddClintCmd {
            base,
        })
    }
}

impl InitingCmd for AddClintCmd {}

impl NeedSysCmd for AddClintCmd {}

impl CosimServerCmd for AddClintCmd {
    fn id(&self) -> CosimCmdId {
        CosimCmdId::AddClint
    }
    fn execute(&self, server: &mut CosimServer) -> Result<(), String> {
        self.check_state(server)?;
        let sys = self.get_sys(server)?;
        sys.register_device("clint", self.base, 0x000c0000, Clint::new(sys.timer())).map_err(|e| { e.to_string() })?;
        Ok(())
    }
}

pub struct AddPlicCmd {
    base: u64,
}

impl CosimCmdTy {
    pub fn add_plic(base:u64) -> CosimCmdTy {
        CosimCmdTy::AddPlic( AddPlicCmd {
            base,
        })
    }
}

impl InitingCmd for AddPlicCmd {}

impl NeedSysCmd for AddPlicCmd {}

impl CosimServerCmd for AddPlicCmd {
    fn id(&self) -> CosimCmdId {
        CosimCmdId::AddPlic
    }
    fn execute(&self, server: &mut CosimServer) -> Result<(), String> {
        self.check_state(server)?;
        let sys = self.get_sys(server)?;
        sys.register_device("plic", self.base, 0x000c0000, Plic::new(sys.intc())).map_err(|e| { e.to_string() })?;
        Ok(())
    }
}

pub struct InitDoneCmd {
    reset_vec: Vec<u64>
}

impl CosimCmdTy {
    pub fn init_done(reset_vec: Vec<u64>) -> CosimCmdTy {
        CosimCmdTy::InitDone( InitDoneCmd {
            reset_vec
        })
    }
}

impl InitingCmd for InitDoneCmd {}

impl NeedSysCmd for InitDoneCmd {}

impl CosimServerCmd for InitDoneCmd {
    fn id(&self) -> CosimCmdId {
        CosimCmdId::InitDone
    }
    fn execute(&self, server: &mut CosimServer) -> Result<(), String> {
        self.check_state(server)?;
        let sys = self.get_sys(server)?;
        sys.reset(self.reset_vec.to_vec()).map_err(|e| { e.to_string() })?;
        server.state = ServerState::Running;
        Ok(())
    }
}

