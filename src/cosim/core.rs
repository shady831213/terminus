use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError, RecvError, SendError};
use std::sync::Mutex;
use std::thread;
use crate::system::System;
use crate::cosim::rapi::*;
use std::num::Wrapping;

struct CosimClient {
    resp: Mutex<Receiver<CosimResp>>,
    cmd: Mutex<(Wrapping<u32>, Sender<CosimCmd>)>,
}

impl CosimClient {
    pub fn send_cmd(&self, ty: CosimCmdTy) -> Result<(), SendError<CosimCmd>> {
        let mut cmd_ch = self.cmd.lock().unwrap();
        let cmd = CosimCmd::new((cmd_ch.0).0, ty);
        cmd_ch.1.send(cmd)?;
        cmd_ch.0 += Wrapping(1);
        Ok(())
    }

    pub fn try_recv_resp(&self) -> Result<CosimResp, TryRecvError> {
        self.resp.lock().unwrap().try_recv()
    }

    pub fn recv_resp(&self) -> Result<CosimResp, RecvError> {
        self.resp.lock().unwrap().recv()
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
    fn run(&mut self) -> Result<CosimResp, String> {
        let cmd = self.cmd.recv().map_err(|e| { e.to_string() })?;
        Ok(CosimResp::new(cmd.meta().clone(), cmd.ty().execute(self)))
    }

    fn serve(&mut self) -> Result<(), String> {
        loop {
            let resp = self.run()?;
            self.resp.send(resp).map_err(|e| { e.to_string() })?;
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
        cmd: Mutex::new((Wrapping(0), cmd_sender)),
    }
}


lazy_static! {
    static ref CLIENT: CosimClient = cosim();
}

pub fn send_cmd(ty: CosimCmdTy) -> Result<(), SendError<CosimCmd>> {
    CLIENT.send_cmd(ty)
}

pub fn try_recv_resp() -> Result<CosimResp, TryRecvError> {
    CLIENT.try_recv_resp()
}

pub fn recv_resp() -> Result<CosimResp, RecvError> {
    CLIENT.recv_resp()
}

#[cfg(test)]
mod test {
    use crate::cosim::core::cosim;
    use crate::cosim::rapi::{CosimCmdTy, CosimRespTy, CosimCmdId, CosimResp};

    #[test]
    fn cosim_sys_init_test() {
        let client = cosim();
        client.send_cmd(CosimCmdTy::reset()).unwrap();
        let resp = client.recv_resp().unwrap();
        println!("{:#x?}",resp);
        client.send_cmd(CosimCmdTy::sys_init("top_tests/elf/rv64ui-p-add", 32)).unwrap();
        let resp = client.recv_resp().unwrap();
        println!("{:#x?}",resp);
        client.send_cmd(CosimCmdTy::init_done(vec![])).unwrap();
        match client.recv_resp() {
            Ok(CosimResp{meta, ty}) => {
                assert_eq!(meta.id, CosimCmdId::InitDone as u32);
                assert_eq!(meta.idx, 2);
                assert_eq!(ty, CosimRespTy::Ok)
            }
            Err(e) => panic!("{:?}", e)
        }
    }

    #[test]
    fn cosim_init_reset_test() {
        let client = cosim();
        client.send_cmd(CosimCmdTy::reset()).unwrap();
        let resp = client.recv_resp().unwrap();
        println!("{:#x?}",resp);
        client.send_cmd(CosimCmdTy::sys_init("top_tests/elf/rv64ui-p-add", 32)).unwrap();
        let resp = client.recv_resp().unwrap();
        println!("{:#x?}",resp);
        client.send_cmd(CosimCmdTy::reset()).unwrap();
        let resp = client.recv_resp().unwrap();
        println!("{:#x?}",resp);
        client.send_cmd(CosimCmdTy::sys_init("top_tests/elf/rv64ui-p-add", 32)).unwrap();
        let resp = client.recv_resp().unwrap();
        println!("{:#x?}",resp);
        client.send_cmd(CosimCmdTy::init_done(vec![])).unwrap();
        match client.recv_resp() {
            Ok(CosimResp{meta, ty}) => {
                assert_eq!(meta.id, CosimCmdId::InitDone as u32);
                assert_eq!(meta.idx, 4);
                assert_eq!(ty, CosimRespTy::Ok)
            }
            Err(e) => panic!("{:?}", e)
        }
    }
}