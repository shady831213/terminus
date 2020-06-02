use crate::system::System;
use crate::cosim::core::{CosimServer, ServerState};

trait CosimServeCmd {
    fn execute(&self, server: &mut CosimServer) -> Result<Option<CosimResp>, String>;
}

trait InitingCmd: CosimServeCmd {
    fn check_state(&self, server: &CosimServer) -> Result<(), String> {
        if server.state == ServerState::Initing {
            Ok(())
        } else {
            Err("init process has been done!".to_string())
        }
    }
}

trait NeedSysCmd: CosimServeCmd {
    fn get_sys<'a>(&self, server: &'a mut CosimServer) -> Result<&'a mut System, String> {
        if let Some(sys) = server.sys.as_mut() {
            Ok(sys)
        } else {
            Err("system not exist!".to_string())
        }
    }
}

trait RunningCmd: CosimServeCmd {
    fn check_state(&self, server: &CosimServer) -> Result<(), String> {
        if server.state == ServerState::Running {
            Ok(())
        } else {
            Err("init process not done!".to_string())
        }
    }
}

pub struct SystemInitCmd {
    elf: String,
    max_int_src: usize,
}

impl SystemInitCmd {
    pub fn new(elf: &str, max_int_src: usize) -> SystemInitCmd {
        SystemInitCmd {
            elf: elf.to_string(),
            max_int_src,
        }
    }
}

impl InitingCmd for SystemInitCmd {}

impl CosimServeCmd for SystemInitCmd {
    fn execute(&self, server: &mut CosimServer) -> Result<Option<CosimResp>, String> {
        self.check_state(server)?;
        if server.sys.is_some() {
            return Err("system has been inited!".to_string());
        }
        server.sys = Some(System::new("cosim_sys", &self.elf, 10000000, self.max_int_src));
        Ok(None)
    }
}

pub struct InitDoneCmd {
    reset_vec: Vec<u64>
}

impl InitDoneCmd {
    pub fn new(reset_vec: Vec<u64>) -> InitDoneCmd {
        InitDoneCmd {
            reset_vec
        }
    }
}

impl InitingCmd for InitDoneCmd {}

impl NeedSysCmd for InitDoneCmd {}

impl CosimServeCmd for InitDoneCmd {
    fn execute(&self, server: &mut CosimServer) -> Result<Option<CosimResp>, String> {
        self.check_state(server)?;
        let sys = self.get_sys(server)?;
        sys.reset(self.reset_vec.to_vec()).map_err(|e| { e.to_string() })?;
        server.state = ServerState::Running;
        Ok(Some(CosimResp::InitOk))
    }
}

pub enum CosimCmd {
    Reset,
    SysInit(SystemInitCmd),
    InitDone(InitDoneCmd),
}

impl CosimCmd {
    pub fn execute(&self, server: &mut CosimServer) -> Option<CosimResp> {
        let res = match self {
            CosimCmd::Reset => {
                server.reset();
                return Some(CosimResp::ResetOk);
            }
            CosimCmd::SysInit(cmd) => cmd.execute(server),
            CosimCmd::InitDone(cmd) => cmd.execute(server),
        };
        match res {
            Ok(resp) => resp,
            Err(e) => Some(CosimResp::Err(e))
        }
    }
}

#[derive(Debug)]
pub enum CosimResp {
    ResetOk,
    InitOk,
    RunOk,
    Err(String),
}
