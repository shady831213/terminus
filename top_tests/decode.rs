 extern crate terminus_macros;
 extern crate terminus_proc_macros;
 extern crate terminus_global;
 extern crate terminus;
 use terminus::*;
 use terminus_global::*;
 use terminus_macros::*;
 use terminus_proc_macros::Instruction;
 use std::thread;
 use std::time::Duration;
 #[derive(Instruction)]
 #[format(B)]
 #[code("0b1??0_1110")]
 #[derive(Debug)]
 struct InsnCodingTestStruct(InsnT);
 impl Execution for InsnCodingTestStruct {
     fn execute(&self, _: &mut Processor) {}
 }
 #[derive(Instruction)]
 #[format(B)]
 #[code("0b1??0_1111")]
 #[derive(Debug)]
 struct InsnCodingTestStruct2(InsnT);
 
 impl Execution for InsnCodingTestStruct2 {
     fn execute(&self, _: &mut Processor) {}
 }
 #[test]
 fn test_decode() {
     let mut threads = vec![];
     for i in 0..3 {
         let p = thread::spawn(move || {
             thread::sleep(Duration::from_millis(3 - i));
             println!("test 1 the spawned thread {}!", i);
             let result = GDECODER.decode(0b1010_1110).unwrap();
             assert_eq!(0b10_1110, result.op());
             assert_eq!(0b1010_1110, result.ir());
             assert_eq!(0b1000_1110, InsnCodingTestStructDecoder.code());
             let mask_bit: InsnT = InsnCodingTestStructDecoder.mask().bit_range(15, 0);
             assert_eq!(0b1001_1111, mask_bit);
             thread::sleep(Duration::from_millis(3 - i));
             println!("test 2 the spawned thread {}!", i);
 
             let result = GDECODER.decode(0b1010_1111).unwrap();
             assert_eq!(0b10_1111, result.op());
             assert_eq!(0b1010_1111, result.ir());
             assert_eq!(0b1000_1111, InsnCodingTestStruct2Decoder.code());
             let mask_bit: InsnT = InsnCodingTestStruct2Decoder.mask().bit_range(15, 0);
             assert_eq!(0b1001_1111, mask_bit);
             thread::sleep(Duration::from_millis(3 - i));
             println!("test 3 the spawned thread {}!", i);
 
             let result = GDECODER.decode(0).err();
             assert_eq!(result, Some(Exception::IllegalInsn(0)))
         });
         threads.push(p);
     }
     for p in threads {
         p.join().unwrap()
     }
 }