use terminus_macros::*;
use terminus_proc_macros::Instruction;
use crate::insn::*;

use std::collections::HashMap;

#[derive(Instruction)]
#[format(B)]
#[code("0b1??0_1110")]
#[derive(Debug)]
struct InsnCodingTestStruct(u32);

impl Execution for InsnCodingTestStruct{
    fn execute(&self){}
}

#[test]
#[ignore]
fn insn_coding_test() {
    let result = GlobalInsnMap::get().decode(0b1010_1110).unwrap();
    assert_eq!(0b10_1110, result.op());
    assert_eq!(0b1010_1110, result.ir());
    assert_eq!(0b1000_1110, InsnCodingTestStructDecoder.code());
    let mask_bit:u32 = InsnCodingTestStructDecoder.mask().bit_range(15,0);
    assert_eq!(0b1001_1111, mask_bit);

    let result = GlobalInsnMap::get().decode(0).err();
    assert_eq!(result, Some("invalid instruction!".to_string()))
}