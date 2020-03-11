use crate::processor::XLen;
use terminus_macros::*;
use std::ops::{Deref, DerefMut};
use super::*;
use terminus_global::RegT;
use terminus_proc_macros::define_csr;


define_csr! {
Test {
    fields {,},
    map {}
}
}

decl_csr! {
    struct Status;
    pub uie, set_uie: 0, 0;
    pub sie, set_sie: 1, 1;
    pub mie, set_mie:3, 3;
    pub upie, set_upie:4, 4;
    pub spie, set_spie:5, 5;
    pub mpie, set_mpie:7, 7;
    pub spp, set_spp:8, 8;
    pub mpp, set_mpp:12, 11;
    pub fs, set_fs:14, 13;
    pub xs, set_xs:16, 15;
    pub mprv, set_mprv:17, 17;
    pub sum, set_sum:18, 18;
    pub mxr, set_mxr:19, 19;
    pub tvm, set_tvm:20, 20;
    pub tw, set_tw:21, 21;
    pub tsr, set_tsr:22, 22;
    pub uxl, set_uxl:33, 32;
    pub sxl, set_sxl:35,34;
    pub sd, set_sd:63, 63;
}

impl CsrAccess for Status {
    fn mask64() -> RegT {
        0b1000_0000_0000_0000_0000_0000_0000_1111_0000_0000_0111_1111_1111_1001_1011_1011
    }
    fn mask32() -> RegT {
        0b0000_0000_0000_0000_0000_0000_0000_0000_1000_0000_0111_1111_1111_1001_1011_1011
    }
    fn write64(&mut self, value: RegT) {
        self.0 = value
    }
    fn read64(&self) -> RegT {
        self.0
    }
    fn write32(&mut self, value: RegT) {
        let mask_low: RegT = Self::mask32().bit_range(30, 0);
        self.0 = value & mask_low;
        self.set_sd(value.bit_range(31, 31));
    }
    fn read32(&self) -> RegT {
        let low: RegT = self.bit_range(30, 0);
        low | (self.sd() << 31)
    }
}

#[test]
fn test_status() {
    let mut status = Status::new(0);
    status.write(XLen::X64, 0x11);
    assert_eq!(status.read(XLen::X64), 0x11);
    assert_eq!(status.uie(), 0x1);
    status.set_uie(0x0);
    assert_eq!(status.read(XLen::X64), 0x10);
}

