use terminus_macros::*;
use terminus_proc_macros::InsnCoding;
#[derive(InsnCoding)]
#[code(0b11_1111)]
#[mask(0xdeadbeaf)]
struct InsnCodingTestStruct{
    ir:u32
}

#[test]
fn insn_coding_test() {
    let item = InsnCodingTestStruct{ir:0b10_1111};
    assert_eq!(0b101111, item.ir());
    assert_eq!(0b111111, item.code());
    let mask_bit:u32 = item.mask().bit_range(31,16);
    assert_eq!(0xdead, mask_bit);
}