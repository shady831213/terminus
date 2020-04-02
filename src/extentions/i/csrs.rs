use terminus_global::*;
use terminus_proc_macros::{define_csr, csr_map};
use terminus_macros::*;
csr_map! {
pub ICsrs(0x0, 0xfff) {
    satp(RW):Satp, 0x180;
    mstatus(RW):MStatus, 0x300;
    pmpcfg0(RW):PmpCfg, 0x3A0;
    pmpcfg1(RW):PmpCfg, 0x3A1;
    pmpcfg2(RW):PmpCfg, 0x3A2;
    pmpcfg3(RW):PmpCfg, 0x3A3;
    pmpaddr0(RW):PmpAddr, 0x3B0;
    pmpaddr1(RW):PmpAddr, 0x3B1;
    pmpaddr2(RW):PmpAddr, 0x3B2;
    pmpaddr3(RW):PmpAddr, 0x3B3;
    pmpaddr4(RW):PmpAddr, 0x3B4;
    pmpaddr5(RW):PmpAddr, 0x3B5;
    pmpaddr6(RW):PmpAddr, 0x3B6;
    pmpaddr7(RW):PmpAddr, 0x3B7;
    pmpaddr8(RW):PmpAddr, 0x3B8;
    pmpaddr9(RW):PmpAddr, 0x3B9;
    pmpaddr10(RW):PmpAddr, 0x3BA;
    pmpaddr11(RW):PmpAddr, 0x3BB;
    pmpaddr12(RW):PmpAddr, 0x3BC;
    pmpaddr13(RW):PmpAddr, 0x3BD;
    pmpaddr14(RW):PmpAddr, 0x3BE;
    pmpaddr15(RW):PmpAddr, 0x3BF;
    mhartid(RO):Mhartid, 0xF14;
}
}

define_csr! {
MStatus {
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

define_csr! {
PmpCfg{
    fields {
        pmpcfg0(RW):7,0;
        pmpcfg1(RW):15,8;
        pmpcfg2(RW):23,16;
        pmpcfg3(RW):31,24;
    },
    fields64{
        pmpcfg4(RW):39,32;
        pmpcfg5(RW):47,40;
        pmpcfg6(RW):55,48;
        pmpcfg7(RW):63,56;
    }
}
}

impl BitRange<u8> for PmpCfg {
    fn bit_range(&self, msb: usize, lsb: usize) -> u8 {
        let width = msb - lsb + 1;
        let mask = (1 << width) - 1;
        ((self.get() >> lsb) & mask) as u8
    }
    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: u8) {
        let width = msb - lsb + 1;
        let mask = !((((1 << width) - 1) << lsb) as RegT);
        self.set((value as RegT) << (lsb as RegT) | self.get() & mask)
    }
}

define_csr! {
PmpAddr {
    fields32 {
        addr(RW):31, 0;
    },
    fields64 {
        addr(RW):53, 0;
    }
}
}

define_csr! {
Satp {
    fields32{
        ppn(RW):21, 0;
        asid(RW):30, 22;
        mode(RW):31,31;
    },
    fields64{
        ppn(RW):43, 0;
        asid(RW):59, 44;
        mode(RW):63, 60;
    }
}
}

define_csr! {
Mhartid{}
}

#[test]
fn test_status() {
    let mut status = MStatus::new(XLen::X32);
    status.set_xs(0xf);
    assert_eq!(status.xs(), 0x3);
    status.set_xs(0);
    assert_eq!(status.xs(), 0);
}

