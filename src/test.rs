use terminus_macros::*;
use terminus_proc_macros::InsnCoding;
use super::insn::{InsnCoding, Format};
#[derive(InsnCoding)]
#[format(B)]
#[code(0b11_1111)]
#[mask(0xdeadbeaf)]
struct InsnCodingTestStruct(u32);
impl InsnCodingTestStruct {
    fn new(ir:u32) -> InsnCodingTestStruct {
        InsnCodingTestStruct(ir)
    }
}

#[test]
fn insn_coding_test() {
    let item = InsnCodingTestStruct::new(0b101111);
    assert_eq!(0b101111, item.op());
    assert_eq!(0b101111, item.ir());
    assert_eq!(0b111111, item.code());
    let mask_bit:u32 = item.mask().bit_range(31,16);
    assert_eq!(0xdead, mask_bit);
}