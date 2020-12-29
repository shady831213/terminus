use crate::prelude::*;
use crate::processor::privilege::m::csrs::{Cycle, Instret, Cycleh, Instreth};
csr_map! {
pub UCsrs(0x0, 0xfff) {
    cycle(RO):Cycle, 0xC00;
    instret(RO):Instret, 0xC02;
    cycleh(RO):Cycleh, 0xC80;
    instreth(RO):Instreth, 0xC82;
}
}
