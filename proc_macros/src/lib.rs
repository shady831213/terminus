extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
#[macro_use]
extern crate terminus_macros;
extern crate regex;

mod insn;

use proc_macro::TokenStream;
use syn::DeriveInput;

///
/// # Example
/// ```rust
/// #
/// extern crate terminus_macros;
/// extern crate terminus_proc_macros;
/// use terminus_macros::*;
/// use terminus_proc_macros::Instruction;
/// pub struct Processor;
/// pub struct Exception;
/// terminus_insn!(u32, Processor, Exception);
/// #[derive(Instruction)]
/// #[format(B)]
/// #[code("8b1??0_1110")]
/// #[derive(Debug)]
/// struct InsnCodingTestStruct();
/// impl Execution for InsnCodingTestStruct {
///     fn execute(&self, p: &mut Processor)->Result<(), Exception> {
///         Ok(())
///     }
/// }
/// #[derive(Instruction)]
/// #[format(B)]
/// #[code("8b1??0_1111")]
/// #[derive(Debug)]
/// struct InsnCodingTestStruct2();
/// 
/// impl Execution for InsnCodingTestStruct2 {
///     fn execute(&self, p: &mut Processor)->Result<(), Exception> {
///         Ok(())
///     }
/// }
///
/// fn main() {
///   let result = GDECODER.decode(&0b1010_1110).unwrap();
///   assert_eq!(0b10_1110, result.op(&0b1010_1110));
///   let result = GDECODER.decode(&0b1010_1111).unwrap();
///   assert_eq!(0b10_1111, result.op(&0b1010_1111));
///   let result = GDECODER.decode(&0).err();
///   assert_eq!(result, Some(Error::Illegal(0)))
/// # }
/// ```
///
#[proc_macro_derive(Instruction, attributes(code, format))]
pub fn instruction(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    match insn::expand(&ast, name) {
        Ok(s) => s.into(),
        Err(e) => e.to_compile_error().into()
    }
}

mod csr;

use syn::parse_macro_input;

/// # Example
/// ```rust
/// #
/// extern crate terminus_macros;
/// extern crate terminus_proc_macros;
/// use terminus_macros::*;
/// use terminus_proc_macros::define_csr;
/// define_csr! {
/// Test {
///     fields {
///          field1(RW): 6, 4;
///     },
///     fields32 {
///          field2(RO): 8, 7;
///     },
///     fields64 {
///          field2(WO): 31, 31;
///          field3(RW): 32, 32;
///     },
/// }
/// }
/// fn main() {
///     let mut test = Test::new(64, 0);
///     test.set_field1_transform(|value|{
///         if value == 1 {
///             3
///         } else {
///             value
///         }
///     });
///     test.set_field1(0x1);
///     assert_eq!(test.field1(), 0x3);
///     test.set_field3(0xff);
///     assert_eq!(test.field3(), 0x1);
///     test.set_field2(3);
///     assert_eq!(test.field2(), 0x1);
///     test.set(0);
///     assert_eq!(test.get(), 0x0);
///     test.set(0xffffffff_ffffffff);
///     assert_eq!(test.get(), 0x1_0000_0070);
///     assert_eq!(test.field2(), 0x1);
///
///     let mut test2 = Test::new(32, 0);
///     test2.set_field1(0xffff_ffff_ffff);
///     assert_eq!(test2.field1(), 0x7);
///     test2.set_field2(0xffff_ffff_ffff);
///     assert_eq!(test2.field2(), 0x3);
///     test2.set_field2(0x0);
///     test2.set(0);
///     assert_eq!(test2.get(), 0x0);
///     test2.set(0xffffffff_ffffffff);
///     assert_eq!(test2.get(), 0x70);
///     test2.set_field2(0xf);
///     assert_eq!(test2.field2(), 0x3);
///     assert_eq!(test2.get(), 0x1f0);
/// # }
/// ```
/// generate code like this:
/// ```rust
/// #
/// extern crate terminus_macros;
/// extern crate terminus_proc_macros;
/// use terminus_macros::*;
/// use terminus_proc_macros::define_csr;
/// #[derive(Copy, Clone)]
/// struct Test32(u32);
/// bitfield_bitrange! {struct Test32(u32)}
/// impl TestTrait for Test32 {
///     fn get(&self) -> u64{
///         (0 as u64) | (self.field1() << (4 as u64)) | (self.field2() << (7 as u64))
///     }
///     fn set(&mut self, value:u64) {
///         self.set_field1(value >> (4 as u64));
///     }
///     bitfield_fields! {
///     u64;
///     field1, set_field1: 6,4;
///     field2, set_field2: 8,7;
///     }
/// }
///
/// #[derive(Copy, Clone)]
/// struct Test64(u64);
/// bitfield_bitrange! {struct Test64(u64)}
/// impl TestTrait for Test64 {
///     fn get(&self) -> u64{
///         (0 as u64) | (self.field1() << (4 as u64)) | (self.field3() << (32 as u64))
///     }
///     fn set(&mut self, value:u64) {
///         self.set_field1(value >> (4 as u64));
///         self.set_field2(value >> (31 as u64));
///         self.set_field3(value >> (32 as u64));
///     }
///     bitfield_fields! {
///     u64;
///     field1, set_field1: 6,4;
///     field2, set_field2: 31,31;
///     field3, set_field3: 32,32;
///     }
/// }
///
/// pub trait TestTrait {
///     fn get(&self) -> u64;
///     fn field1(&self) -> u64 { panic!("not implemnt") }
///     fn field2(&self) -> u64 { panic!("not implemnt") }
///     fn field3(&self) -> u64 { panic!("not implemnt") }
///     fn set(&mut self, value:u64);
///     fn set_field1(&mut self, value: u64) { panic!("not implemnt") }
///     fn set_field2(&mut self, value: u64) { panic!("not implemnt") }
///     fn set_field3(&mut self, value: u64) { panic!("not implemnt") }
/// }
///
/// union TestU {
///     x32: Test32,
///     x64: Test64,
/// }
///
/// struct Test {
///     pub xlen: usize,
///     csr: TestU,
/// }
///
/// impl Test {
///     pub fn new(xlen:usize, init:u64) -> Test {
///         Test {
///             xlen,
///             csr:TestU{x64:Test64(init)}
///         }
///     }
/// }
/// impl TestTrait for Test {
///     fn get(&self) -> u64 {
///         match self.xlen {
///             64 => unsafe { self.csr.x64.get() },
///             32 => unsafe { self.csr.x32.get() },
///             _ => unreachable!()
///         }
///     }
///     fn set(&mut self, value:u64) {
///         match self.xlen {
///             64 => unsafe { self.csr.x64.set(value) },
///             32 => unsafe { self.csr.x32.set(value) },
///             _ => unreachable!()
///         }
///     }
///     fn field1(&self) -> u64 {
///         match self.xlen {
///             64 => unsafe { self.csr.x64.field1() },
///             32 => unsafe { self.csr.x32.field1() },
///             _ => unreachable!()
///         }
///     }
///     fn field2(&self) -> u64 {
///         match self.xlen {
///             64 => unsafe { self.csr.x64.field2() },
///             32 => unsafe { self.csr.x32.field2() },
///             _ => unreachable!()
///         }
///     }
///     fn field3(&self) -> u64 {
///         match self.xlen {
///             64 => unsafe { self.csr.x64.field3() },
///             32 => unsafe { self.csr.x32.field3() },
///             _ => unreachable!()
///         }
///     }
///     fn set_field1(&mut self, value: u64) {
///         match self.xlen {
///             64 => unsafe { self.csr.x64.set_field1(value) },
///             32 => unsafe { self.csr.x32.set_field1(value) },
///             _ => unreachable!()
///         }
///     }
///     fn set_field2(&mut self, value: u64) {
///         match self.xlen {
///             64 => unsafe { self.csr.x64.set_field2(value) },
///             32 => unsafe { self.csr.x32.set_field2(value) },
///             _ => unreachable!()
///         }
///     }
///     fn set_field3(&mut self, value: u64) {
///         match self.xlen {
///             64 => unsafe { self.csr.x64.set_field3(value.into()) },
///             32 => unsafe { self.csr.x32.set_field3(value.into()) },
///             _ => unreachable!()
///         }
///     }
/// }
/// fn main() {
///     let mut test = Test::new(64, 0);
///     test.set_field3(0xff);
///     assert_eq!(test.field3(), 0x1);
///     test.set_field2(3);
///     assert_eq!(test.field2(), 0x1);
///     test.set(0);
///     assert_eq!(test.get(), 0x0);
///     test.set(0xffffffff_ffffffff);
///     assert_eq!(test.get(), 0x1_0000_0070);
///     assert_eq!(test.field2(), 0x1);
///
///     let mut test2 = Test::new(32, 0);
///     test2.set_field1(0xffff_ffff_ffff);
///     assert_eq!(test2.field1(), 0x7);
///     test2.set_field2(0xffff_ffff_ffff);
///     assert_eq!(test2.field2(), 0x3);
///     test2.set_field2(0x0);
///     test2.set(0);
///     assert_eq!(test2.get(), 0x0);
///     test2.set(0xffffffff_ffffffff);
///     assert_eq!(test2.get(), 0x70);
///     test2.set_field2(0xf);
///     assert_eq!(test2.field2(), 0x3);
///     assert_eq!(test2.get(), 0x1f0);
/// # }
/// ```
#[proc_macro]
pub fn define_csr(input: TokenStream) -> TokenStream {
    csr::define_csr::expand(parse_macro_input!(input)).into()
}

/// # Example
/// ```rust
/// #
/// extern crate terminus_macros;
/// extern crate terminus_proc_macros;
/// use terminus_macros::*;
/// use terminus_proc_macros::{define_csr, csr_map};
/// define_csr! {
/// Test {
///     fields {
///          field1(RW): 6, 4;
///     },
///     fields32 {
///          field2(RO): 8, 7;
///     },
///     fields64 {
///          field2(WO): 31, 31;
///          field3(RW): 32, 32;
///     },
/// }
/// }
///
/// csr_map! {
/// pub CSR(0x0, 0xa) {
///     test1(RW):Test, 0x1;
///     test2(RW):Test, 0x7;
/// }
/// }
/// csr_map! {
/// CSR_p(0x0, 0xa) {
///     test1(WO):Test, 0x1;
///     test2(RO):Test, 0x7;
/// }
/// }
/// fn main(){
///     let csr = CSR::new(64);
///     csr.write(1, 0xffffffff_ffffffff);
///     assert_eq!(csr.test1().get(), 0x1_0000_0070);
///     assert_eq!(csr.test1().field2(), 0x1);
///     assert_eq!(csr.read(1).unwrap(), 0x1_0000_0070);
///     assert_eq!(csr.read(0xb), None);
///     let csr_p = CSR_p::new(32);
///
///     csr_p.write(7, 0xffffffff_ffffffff);
///     csr_p.write(1, 0xffffffff_ffffffff);
///     assert_eq!(csr_p.read(7).unwrap(), 0);
///     assert_eq!(csr_p.test2().get(), 0x0);
///     assert_eq!(csr_p.read(1).unwrap(), 0);
///     csr_p.test2_mut().set( 0xffffffff_ffffffff);
///     assert_eq!(csr_p.read(7).unwrap(), 0x70);
///     assert_eq!(csr_p.read(1).unwrap(), 0);
///     assert_eq!(csr_p.test1().get(), 0x70);
///     assert_eq!(csr_p.write(2, 0), None);
/// # }
///
/// ```
#[proc_macro]
pub fn csr_map(input: TokenStream) -> TokenStream {
    csr::csr_map::expand(parse_macro_input!(input)).into()
}


