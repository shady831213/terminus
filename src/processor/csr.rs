use crate::processor::XLen;
use terminus_macros::*;
use std::ops::{Deref, DerefMut};
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

#[derive(Copy, Clone)]
struct Test32(u32);
bitfield_bitrange! {struct Test32(u32)}
impl TestTrait for Test32 {
    bitfield_fields! {
    RegT;
    field1, set_field1: 6,4;
    field2, set_field2: 7,7;
    }
}

#[derive(Copy, Clone)]
struct Test64(u64);
bitfield_bitrange! {struct Test64(u64)}
impl TestTrait for Test64 {
    bitfield_fields! {
    RegT;
    field1, set_field1: 6,4;
    field2, set_field2: 31,31;
    field3, set_field3: 32,32;
    }
}

trait TestTrait {
    fn field1(&self) -> RegT { panic!("not implemnt") }
    fn field2(&self) -> RegT { panic!("not implemnt") }
    fn field3(&self) -> RegT { panic!("not implemnt") }
    fn set_field1(&mut self, value: RegT) { panic!("not implemnt") }
    fn set_field2(&mut self, value: RegT) { panic!("not implemnt") }
    fn set_field3(&mut self, value: RegT) { panic!("not implemnt") }
}

union TestU {
    x32: Test32,
    x64: Test64,
}

struct Test {
    xlen: XLen,
    csr: TestU,
}

impl TestTrait for Test {
    fn field1(&self) -> RegT {
        match self.xlen {
            XLen::X64 => unsafe { self.csr.x64.field1() },
            XLen::X32 => unsafe { self.csr.x32.field1() }
        }
    }
    fn field2(&self) -> RegT {
        match self.xlen {
            XLen::X64 => unsafe { self.csr.x64.field2() },
            XLen::X32 => unsafe { self.csr.x32.field2() }
        }
    }
    fn field3(&self) -> RegT {
        match self.xlen {
            XLen::X64 => unsafe { self.csr.x64.field3() },
            XLen::X32 => unsafe { self.csr.x32.field3() }
        }
    }
    fn set_field1(&mut self, value: RegT) {
        match self.xlen {
            XLen::X64 => unsafe { self.csr.x64.set_field1(value) },
            XLen::X32 => unsafe { self.csr.x32.set_field1(value) }
        }
    }
    fn set_field2(&mut self, value: RegT) {
        match self.xlen {
            XLen::X64 => unsafe { self.csr.x64.set_field2(value) },
            XLen::X32 => unsafe { self.csr.x32.set_field2(value) }
        }
    }
    fn set_field3(&mut self, value: RegT) {
        match self.xlen {
            XLen::X64 => unsafe { self.csr.x64.set_field3(value.into()) },
            XLen::X32 => unsafe { self.csr.x32.set_field3(value.into()) }
        }
    }
}


#[test]
fn test_status() {
    let mut test = Test {
        xlen: XLen::X64,
        csr: TestU {
            x64: Test64(0)
        },
    };

    test.set_field3(0xff);
    println!("x64 field3 {:x}", test.field3());
    test.set_field2(3);

    let mut test2 = Test {
        xlen: XLen::X32,
        csr: TestU {
            x64: Test64(0)
        },
    };
    &test2.set_field1(0xffff_ffff_ffff);
    println!("x32 field1 {:x}", &test2.field1());
    println!("x32 field2 {:x}", &test2.field2());


    let mut status = Status::new(0);
    status.write(XLen::X64, 0x11);
    assert_eq!(status.read(XLen::X64), 0x11);
    assert_eq!(status.uie(), 0x1);
    status.set_uie(0x0);
    assert_eq!(status.read(XLen::X64), 0x10);
}

