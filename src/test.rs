use terminus_macros::*;
use terminus_proc_macros::Instruction;
use super::insn::*;
use super::*;
use super::execption::*;
use super::decode::*;

#[derive(Instruction)]
#[format(B)]
#[code("0b1??0_1110")]
#[derive(Debug)]
struct InsnCodingTestStruct(u32);

impl Execution for InsnCodingTestStruct{
    fn execute(&self){}
}

#[derive(Instruction)]
#[format(B)]
#[code("0b1??0_1111")]
#[derive(Debug)]
struct InsnCodingTestStruct2(u32);

impl Execution for InsnCodingTestStruct2{
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

    let result = GlobalInsnMap::get().decode(0b1010_1111).unwrap();
    assert_eq!(0b10_1111, result.op());
    assert_eq!(0b1010_1111, result.ir());
    assert_eq!(0b1000_1111, InsnCodingTestStruct2Decoder.code());
    let mask_bit:u32 = InsnCodingTestStruct2Decoder.mask().bit_range(15,0);
    assert_eq!(0b1001_1111, mask_bit);

    let result = GlobalInsnMap::get().decode(0).err();
    assert_eq!(result, Some(Exception::IllegalInsn(0)))
}