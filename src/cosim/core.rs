use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError, RecvError, SendError};
use std::sync::Mutex;
use std::thread;
use crate::system::System;

trait CosimServeCmd {
    fn execute(&self, server: &mut CosimServer) -> Result<(), String>;
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
    fn execute(&self, server: &mut CosimServer) -> Result<(), String> {
        self.check_state(server)?;
        if server.sys.is_some() {
            return Err("system has been inited!".to_string());
        }
        server.sys = Some(System::new("cosim_sys", &self.elf, 10000000, self.max_int_src));
        Ok(())
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
    fn execute(&self, server: &mut CosimServer) -> Result<(), String> {
        self.check_state(server)?;
        let sys = self.get_sys(server)?;
        sys.reset(self.reset_vec.to_vec()).map_err(|e| { e.to_string() })?;
        server.state = ServerState::Running;
        Ok(())
    }
}

pub enum CosimCmd {
    SysInit(SystemInitCmd),
    InitDone(InitDoneCmd),
}

pub enum CosimResp {
    InitOk,
    InitErr(String),
}


struct CosimClient {
    resp: Mutex<Receiver<CosimResp>>,
    cmd: Mutex<Sender<CosimCmd>>,
}

impl CosimClient {
    fn cmd(&self) -> &Mutex<Sender<CosimCmd>> {
        &self.cmd
    }
    fn resp(&self) -> &Mutex<Receiver<CosimResp>> {
        &self.resp
    }
}

#[derive(Eq, PartialEq)]
enum ServerState {
    Initing,
    Running,
}

struct CosimServer {
    sys: Option<System>,
    state: ServerState,
    resp: Sender<CosimResp>,
    cmd: Receiver<CosimCmd>,
}

impl CosimServer {
    fn init(&mut self) -> Result<(), String> {
        if self.state != ServerState::Initing {
            return Err("server has been inited!".to_string());
        }
        while self.state == ServerState::Initing {
            let cmd = self.cmd.recv().map_err(|e| { e.to_string() })?;
            match cmd {
                CosimCmd::SysInit(cmd) => cmd.execute(self)?,
                CosimCmd::InitDone(cmd) => cmd.execute(self)?
            }
        }
        Ok(())
    }
}


fn cosim() -> CosimClient {
    let (cmd_sender, cmd_receiver) = channel();
    let (resp_sender, resp_receiver) = channel();
    thread::Builder::new()
        .name("CosimServer".into())
        .spawn(move || {
            let mut server = CosimServer {
                sys: None,
                state: ServerState::Initing,
                resp: resp_sender,
                cmd: cmd_receiver,
            };
            if let Err(e) = server.init() {
                server.resp.send(CosimResp::InitErr(e)).unwrap();
                return;
            }
            server.resp.send(CosimResp::InitOk).unwrap();
        })
        .expect("failed to start cosim server");
    CosimClient {
        resp: Mutex::new(resp_receiver),
        cmd: Mutex::new(cmd_sender),
    }
}


lazy_static! {
    static ref CLIENT: CosimClient = cosim();
}

pub fn send_cmd(cmd: CosimCmd) -> Result<(), SendError<CosimCmd>> {
    CLIENT.cmd().lock().unwrap().send(cmd)
}

pub fn try_recv_resp() -> Result<CosimResp, TryRecvError> {
    CLIENT.resp().lock().unwrap().try_recv()
}

pub fn recv_resp() -> Result<CosimResp, RecvError> {
    CLIENT.resp().lock().unwrap().recv()
}