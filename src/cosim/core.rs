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
    Reset,
    Finish,
    SysInit(SystemInitCmd),
    InitDone(InitDoneCmd),
}

#[derive(Debug)]
pub enum CosimResp {
    ResetOk,
    InitOk,
    RunOk,
    FinishOk,
    Err(String),
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
    fn reset(&mut self) {
        self.sys = None;
        self.state = ServerState::Initing;
    }
    fn run(&mut self) -> Option<CosimResp> {
        let result = match self.cmd.recv().map_err(|e| { e.to_string() }) {
            Ok(cmd) => {
                match cmd {
                    CosimCmd::Reset => {
                        self.reset();
                        return Some(CosimResp::ResetOk);
                    }
                    CosimCmd::Finish => {
                        return Some(CosimResp::FinishOk);
                    }
                    CosimCmd::SysInit(cmd) => cmd.execute(self),
                    CosimCmd::InitDone(cmd) => {
                        let res = cmd.execute(self);
                        if res.is_ok() {
                            return Some(CosimResp::InitOk);
                        }
                        res
                    }
                    _ => Err("Illegal cmd!".to_string())
                }
            }
            Err(e) => Err(e)
        };
        self.result_to_resp(result)
    }

    fn serve(&mut self) {
        loop {
            if let Some(resp) = self.run() {
                if let CosimResp::FinishOk = resp {
                    return;
                } else {
                    self.resp.send(resp).unwrap();
                }
            }
        }
    }

    fn result_to_resp(&self, res: Result<(), String>) -> Option<CosimResp> {
        if let Err(e) = res {
            Some(CosimResp::Err(e))
        } else {
            match self.state {
                ServerState::Initing => {
                    None
                }
                ServerState::Running => {
                    Some(CosimResp::RunOk)
                }
            }
        }
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
            server.serve()
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

#[cfg(test)]
mod test {
    use crate::cosim::core::{cosim, CosimCmd, SystemInitCmd, InitDoneCmd, CosimResp};

    #[test]
    fn cosim_sys_init_test() {
        let client = cosim();
        let (cmd, resp) = (client.cmd.lock().unwrap(), client.resp.lock().unwrap());
        cmd.send(CosimCmd::Reset).unwrap();
        resp.recv().unwrap();
        cmd.send(CosimCmd::SysInit(SystemInitCmd::new("top_tests/elf/rv64ui-p-add", 32))).unwrap();
        cmd.send(CosimCmd::InitDone(InitDoneCmd::new(vec![]))).unwrap();
        match resp.recv() {
            Ok(resp) => {
                match resp {
                    CosimResp::InitOk => { println!("ok!") }
                    _ => panic!("expect initOk but get {:?}", resp)
                }
            }
            Err(e) => panic!("{:?}", e)
        }
        cmd.send(CosimCmd::Finish).unwrap();
    }

    #[test]
    fn cosim_init_reset_test() {
        let client = cosim();
        let (cmd, resp) = (client.cmd.lock().unwrap(), client.resp.lock().unwrap());
        cmd.send(CosimCmd::Reset).unwrap();
        resp.recv().unwrap();
        cmd.send(CosimCmd::SysInit(SystemInitCmd::new("top_tests/elf/rv64ui-p-add", 32))).unwrap();
        cmd.send(CosimCmd::Reset).unwrap();
        resp.recv().unwrap();
        cmd.send(CosimCmd::SysInit(SystemInitCmd::new("top_tests/elf/rv64ui-p-add", 32))).unwrap();
        cmd.send(CosimCmd::InitDone(InitDoneCmd::new(vec![]))).unwrap();
        match resp.recv() {
            Ok(resp) => {
                match resp {
                    CosimResp::InitOk => { println!("ok!") }
                    _ => panic!("expect initOk but get {:?}", resp)
                }
            }
            Err(e) => panic!("{:?}", e)
        }
        cmd.send(CosimCmd::Finish).unwrap();
    }
}