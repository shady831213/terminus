use crate::processor::RegT;
use terminus_macros::*;
use std::ops::{Deref, DerefMut};

decl_csr! {
    struct Status(RegT);
    impl Debug;
    pub uie, set_uie: 0, 0;
}

#[test]
fn test_status() {
    let mut status = Status(0);
    status.write(0x11);
    assert_eq!(status.read(), 0x11);
    assert_eq!(status.uie(), 0x1);
    status.set_uie(0x0);
    assert_eq!(status.read(), 0x10);
}

