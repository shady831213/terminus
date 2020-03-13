extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
#[macro_use]
extern crate lazy_static;
extern crate regex;

mod insn;

use proc_macro::TokenStream;
use syn::DeriveInput;

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
/// extern crate terminus_global;
/// use terminus_global::*;
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
///     let mut test = Test::new(XLen::X64);
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
///     let mut test2 = Test::new(XLen::X32);
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
/// extern crate terminus_global;
/// use terminus_global::*;
/// use terminus_macros::*;
/// use terminus_proc_macros::define_csr;
/// #[derive(Copy, Clone)]
/// struct Test32(u32);
/// bitfield_bitrange! {struct Test32(u32)}
/// impl TestTrait for Test32 {
///     fn get(&self) -> RegT{
///         (0 as RegT) | (self.field1() << (4 as RegT)) | (self.field2() << (7 as RegT))
///     }
///     fn set(&mut self, value:RegT) {
///         self.set_field1(value >> (4 as RegT));
///     }
///     bitfield_fields! {
///     RegT;
///     field1, set_field1: 6,4;
///     field2, set_field2: 8,7;
///     }
/// }
///
/// #[derive(Copy, Clone)]
/// struct Test64(u64);
/// bitfield_bitrange! {struct Test64(u64)}
/// impl TestTrait for Test64 {
///     fn get(&self) -> RegT{
///         (0 as RegT) | (self.field1() << (4 as RegT)) | (self.field3() << (32 as RegT))
///     }
///     fn set(&mut self, value:RegT) {
///         self.set_field1(value >> (4 as RegT));
///         self.set_field2(value >> (31 as RegT));
///         self.set_field3(value >> (32 as RegT));
///     }
///     bitfield_fields! {
///     RegT;
///     field1, set_field1: 6,4;
///     field2, set_field2: 31,31;
///     field3, set_field3: 32,32;
///     }
/// }
///
/// pub trait TestTrait {
///     fn get(&self) -> RegT;
///     fn field1(&self) -> RegT { panic!("not implemnt") }
///     fn field2(&self) -> RegT { panic!("not implemnt") }
///     fn field3(&self) -> RegT { panic!("not implemnt") }
///     fn set(&mut self, value:RegT);
///     fn set_field1(&mut self, value: RegT) { panic!("not implemnt") }
///     fn set_field2(&mut self, value: RegT) { panic!("not implemnt") }
///     fn set_field3(&mut self, value: RegT) { panic!("not implemnt") }
/// }
///
/// union TestU {
///     x32: Test32,
///     x64: Test64,
/// }
///
/// struct Test {
///     xlen: XLen,
///     csr: TestU,
/// }
///
/// impl Test {
///     pub fn new(xlen:XLen) -> Test {
///         Test {
///             xlen,
///             csr:TestU{x64:Test64(0)}
///         }
///     }
/// }
/// impl TestTrait for Test {
///     fn get(&self) -> RegT {
///         match self.xlen {
///             XLen::X64 => unsafe { self.csr.x64.get() },
///             XLen::X32 => unsafe { self.csr.x32.get() }
///         }
///     }
///     fn set(&mut self, value:RegT) {
///         match self.xlen {
///             XLen::X64 => unsafe { self.csr.x64.set(value) },
///             XLen::X32 => unsafe { self.csr.x32.set(value) }
///         }
///     }
///     fn field1(&self) -> RegT {
///         match self.xlen {
///             XLen::X64 => unsafe { self.csr.x64.field1() },
///             XLen::X32 => unsafe { self.csr.x32.field1() }
///         }
///     }
///     fn field2(&self) -> RegT {
///         match self.xlen {
///             XLen::X64 => unsafe { self.csr.x64.field2() },
///             XLen::X32 => unsafe { self.csr.x32.field2() }
///         }
///     }
///     fn field3(&self) -> RegT {
///         match self.xlen {
///             XLen::X64 => unsafe { self.csr.x64.field3() },
///             XLen::X32 => unsafe { self.csr.x32.field3() }
///         }
///     }
///     fn set_field1(&mut self, value: RegT) {
///         match self.xlen {
///             XLen::X64 => unsafe { self.csr.x64.set_field1(value) },
///             XLen::X32 => unsafe { self.csr.x32.set_field1(value) }
///         }
///     }
///     fn set_field2(&mut self, value: RegT) {
///         match self.xlen {
///             XLen::X64 => unsafe { self.csr.x64.set_field2(value) },
///             XLen::X32 => unsafe { self.csr.x32.set_field2(value) }
///         }
///     }
///     fn set_field3(&mut self, value: RegT) {
///         match self.xlen {
///             XLen::X64 => unsafe { self.csr.x64.set_field3(value.into()) },
///             XLen::X32 => unsafe { self.csr.x32.set_field3(value.into()) }
///         }
///     }
/// }
/// fn main() {
///     let mut test = Test::new(XLen::X64);
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
///     let mut test2 = Test::new(XLen::X32);
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
    csr::expand(parse_macro_input!(input)).into()
}