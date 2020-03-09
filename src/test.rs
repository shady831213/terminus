use terminus_macros::*;
use terminus_proc_macros::Instruction;
use super::insn::{Decode, Format};
#[derive(Instruction)]
#[format(B)]
#[code("0b1??0_1110")]
struct InsnCodingTestStruct(u32);
impl InsnCodingTestStruct {
    fn new(ir:u32) -> InsnCodingTestStruct {
        InsnCodingTestStruct(ir)
    }
}

#[test]
fn insn_coding_test() {
    let item = InsnCodingTestStruct::new(0b1010_1110);
    assert_eq!(0b10_1110, item.op());
    assert_eq!(0b1010_1110, item.ir());
    assert_eq!(0b1000_1110, InsnCodingTestStruct::code());
    let mask_bit:u32 = InsnCodingTestStruct::mask().bit_range(15,0);
    assert_eq!(0b1001_1111, mask_bit);
}