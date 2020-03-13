use terminus_macros::*;
use terminus_global::*;
use super::*;
use terminus_global::RegT;
use terminus_proc_macros::define_csr;
use std::rc::Rc;


define_csr! {
Status {
    fields {
         uie(RW): 0, 0;
         sie(RW): 1, 1;
         mie(RW): 3, 3;
         upie(RW): 4, 4;
         spie(RW): 5, 5;
         mpie(RW): 7, 7;
         spp(RW): 8, 8;
         mpp(RW): 12, 11;
         fs(RW): 14, 13;
         xs(RW): 16, 15;
         mprv(RW): 17, 17;
         sum(RW): 18, 18;
         mxr(RW): 19, 19;
         tvm(RW): 20, 20;
         tw(RW): 21, 21;
         tsr(RW): 22, 22;
    },
    fields32 {
         sd(RW): 31, 31;
    },
    fields64 {
         uxl(RW): 33, 32;
         sxl(RW): 35,34;
         sd(RW): 63, 63;
    },
}
}

#[test]
fn test_status() {
    let mut status = Status::new(XLen::X32);
    status.set_xs(0xf);
    assert_eq!(status.xs(), 0x3);
    status.set_xs(0);
    assert_eq!(status.xs(), 0);
}

