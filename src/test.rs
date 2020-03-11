use terminus_macros::*;
use terminus_proc_macros::Instruction;
use super::insn::*;
use super::*;
use super::execption::*;
use super::decode::*;
use crate::processor::Processor;
use terminus_global::InsnT;

#[derive(Instruction)]
#[format(B)]
#[code("0b1??0_1110")]
#[derive(Debug)]
struct InsnCodingTestStruct(InsnT);

impl Execution for InsnCodingTestStruct {
    fn execute(&self, p: &mut Processor) {}
}

#[derive(Instruction)]
#[format(B)]
#[code("0b1??0_1111")]
#[derive(Debug)]
struct InsnCodingTestStruct2(InsnT);

impl Execution for InsnCodingTestStruct2 {
    fn execute(&self,  p: &mut Processor) {}
}

#[test]
#[ignore]
fn insn_coding_test() {
    let result = GlobalInsnMap::get().decode(0b1010_1110).unwrap();
    assert_eq!(0b10_1110, result.op());
    assert_eq!(0b1010_1110, result.ir());
    assert_eq!(0b1000_1110, InsnCodingTestStructDecoder.code());
    let mask_bit: InsnT = InsnCodingTestStructDecoder.mask().bit_range(15, 0);
    assert_eq!(0b1001_1111, mask_bit);

    let result = GlobalInsnMap::get().decode(0b1010_1111).unwrap();
    assert_eq!(0b10_1111, result.op());
    assert_eq!(0b1010_1111, result.ir());
    assert_eq!(0b1000_1111, InsnCodingTestStruct2Decoder.code());
    let mask_bit: InsnT = InsnCodingTestStruct2Decoder.mask().bit_range(15, 0);
    assert_eq!(0b1001_1111, mask_bit);

    let result = GlobalInsnMap::get().decode(0).err();
    assert_eq!(result, Some(Exception::IllegalInsn(0)))
}