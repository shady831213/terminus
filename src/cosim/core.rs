use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError, SendError};
use std::sync::Mutex;
use std::thread;

enum CosimCmd {
    Init(String, usize)
}

enum CosimResp {
    Ok,
    Err,
}


struct CosimClient {
    resp: Mutex<Receiver<CosimResp>>,
    cmd: Mutex<Sender<CosimCmd>>,
}

struct CosimServer {
    resp: Sender<CosimResp>,
    cmd: Receiver<CosimCmd>,
}

impl CosimServer {}


fn cosim() -> CosimClient {
    let (cmd_sender, cmd_receiver) = channel();
    let (resp_sender, resp_receiver) = channel();
    thread::Builder::new()
        .name("CosimServer".into())
        .spawn(move || {
            let server = CosimServer {
                resp: resp_sender,
                cmd: cmd_receiver,
            };
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