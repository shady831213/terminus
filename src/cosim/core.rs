use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError, RecvError, SendError};
use std::sync::Mutex;
use std::thread;
use crate::system::System;
use crate::cosim::rapi::*;

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
pub enum ServerState {
    Initing,
    Running,
}

pub struct CosimServer {
    pub sys: Option<System>,
    pub state: ServerState,
    resp: Sender<CosimResp>,
    cmd: Receiver<CosimCmd>,
}

impl CosimServer {
    pub fn reset(&mut self) {
        self.sys = None;
        self.state = ServerState::Initing;
    }
    fn run(&mut self) -> Result<Option<CosimResp>, String> {
        Ok(self.cmd.recv().map_err(|e|{e.to_string()})?.execute(self))
    }

    fn serve(&mut self) -> Result<(), String>{
        loop {
            if let Some(resp) = self.run()? {
                self.resp.send(resp).map_err(|e|{e.to_string()})?;
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
                    CosimResp::InitOk => {println!("ok!")}
                    _ => panic!("expect initOk but get {:?}", resp)
                }
            }
            Err(e) => panic!("{:?}", e)
        }
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
                    CosimResp::InitOk => {println!("ok!")}
                    _ => panic!("expect initOk but get {:?}", resp)
                }
            }
            Err(e) => panic!("{:?}", e)
        }
    }
}