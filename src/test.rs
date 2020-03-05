use terminus_macros::*;
use terminus_proc_macros::InsnCoding;
#[derive(InsnCoding)]
#[code(0b111111)]
#[mask(0xdeadbeaf)]
struct InsnCodingTestStruct{
}

#[test]
fn insn_coding_test() {
    let item = InsnCodingTestStruct{};
    assert_eq!(0b111111, item.code());
    let mask_bit:u32 = item.mask().bit_range(31,16);
    assert_eq!(0xdead, mask_bit);
}