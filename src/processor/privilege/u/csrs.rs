use crate::prelude::*;
use crate::processor::privilege::m::csrs::*;
csr_map! {
pub UCsrs(0x0, 0xfff) {
    cycle(RO):Cycle, 0xC00;
    instret(RO):Instret, 0xC02;
    cycleh(RO):Cycle, 0xC80;
    instreth(RO):Instret, 0xC82;
}
}
