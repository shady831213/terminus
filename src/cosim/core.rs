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
    resp: Receiver<CosimResp>,
    cmd: Sender<CosimCmd>,
}

struct CosimServer {
    resp: Sender<CosimResp>,
    cmd: Receiver<CosimCmd>,
}

impl CosimServer {}


fn cosim() -> Mutex<CosimClient> {
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
        .expect("failed to spawn thread");
    Mutex::new(CosimClient {
        resp: resp_receiver,
        cmd: cmd_sender,
    })
}


lazy_static! {
    static ref CLIENT: Mutex<CosimClient> = cosim();
}