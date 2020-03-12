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
#[proc_macro]
pub fn define_csr(input: TokenStream) -> TokenStream {
    csr::expand(parse_macro_input!(input)).into()
}