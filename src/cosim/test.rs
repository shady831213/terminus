use crate::cosim::core::{send_cmd, try_recv_resp, recv_resp, CosimCmd, SystemInitCmd, InitDoneCmd, CosimResp};

#[test]
fn sys_init_test() {
    send_cmd(CosimCmd::SysInit(SystemInitCmd::new("top_tests/elf/rv64ui-p-add", 32))).unwrap();
    send_cmd(CosimCmd::InitDone(InitDoneCmd::new(vec![]))).unwrap();
    match recv_resp() {
        Ok(resp) => {
            match resp {
                CosimResp::InitErr(s) => panic!("{}", s),
                _ => {println!("ok!")}
            }
        }
        Err(e) => panic!("{:?}", e)
    }
}