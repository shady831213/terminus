use crate::prelude::*;
use crate::processor::extensions::i::csrs::{Tvec, Scratch, Epc, Cause, Tval, Counteren};
csr_map! {
pub SCsrs(0x0, 0xfff) {
    sstatus(RW):SStatus,0x100;
    sie(RW):Sie, 0x104;
    stvec(RW):Tvec, 0x105;
    scounteren(RW):Counteren, 0x106;
    sscratch(RW):Scratch, 0x140;
    sepc(RW):Epc, 0x141;
    scause(RW):Cause, 0x142;
    stval(RW):Tval, 0x143;
    sip(RW):Sip, 0x144;
    satp(RW):Satp, 0x180;
}
}

define_csr! {
SStatus {
    fields {
         uie(RW): 0, 0;
         sie(RW): 1, 1;
         upie(RW): 4, 4;
         spie(RW): 5, 5;
         spp(RW): 8, 8;
         fs(RW): 14, 13;
         xs(RW): 16, 15;
         sum(RW): 18, 18;
         mxr(RW): 19, 19;
    },
    fields32 {
         sd(RO): 31, 31;
    },
    fields64 {
         uxl(RO): 33, 32;
         sd(RO): 63, 63;
    },
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
Sip {
    fields{
        usip(RO):0,0;
        ssip(RW):1,1;
        utip(RO):4,4;
        stip(RO):5,5;
        ueip(RO):8,8;
        seip(RO):9,9;
    }
}
}

define_csr! {
Sie {
    fields{
        usie(RW):0,0;
        ssie(RW):1,1;
        utie(RW):4,4;
        stie(RW):5,5;
        ueie(RW):8,8;
        seie(RW):9,9;
    }
}
}