extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate terminus_macros;

use proc_macro::TokenStream;
use syn::{DeriveInput, DataStruct, Ident};
use proc_macro2::Span;
use terminus_macros::*;

#[proc_macro_derive(InsnCoding, attributes(code, mask))]
pub fn insn_coding(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    match insn_codeing_transform(&ast, name) {
        Ok(s) => s.into(),
        Err(e) => e.to_compile_error().into()
    }
}

fn insn_codeing_transform(ast: &DeriveInput, name:&Ident) -> Result<proc_macro2::TokenStream, syn::parse::Error> {
    if let syn::Data::Struct(data) = &ast.data {
        let code = parse_int_attr(ast, "code")?;
        let mask = parse_int_attr(ast, "mask")?;
        check_fields(data)?;
        Ok(quote!(
            impl InsnCoding for #name {
                fn ir(&self) ->  u32 {
                    if (self.ir & self.mask() != self.code() & self.mask()) {
                        panic!(format!("maskd ir 0x{:x}, masked code 0x{:x} are not match! ir 0x{:x},  code 0x{:x},  mask 0x{:x}", self.ir & self.mask(), self.code() & self.mask(), self.ir, self.code(), self.mask()))
                    }
                    self.ir
                }
                fn code(&self) ->  u32 {
                    #code
                }
                fn mask(&self) ->  u32 {
                    #mask
                }
            }
        ))
    } else {
        Err(syn::parse::Error::new(Span::call_site(), "Only Struct can derive"))
    }
}

fn check_fields(data: &DataStruct) -> Result<bool, syn::parse::Error> {
    if let Some(_) = data.fields.iter().find(|filed| {
        if filed.ident != Some(syn::Ident::new("ir", proc_macro2::Span::call_site())) {
            return false;
        }
        if let syn::Type::Path(ref path) = filed.ty {
            if path.path.segments.len() != 1 || path.path.segments[0].ident != syn::Ident::new("u32", proc_macro2::Span::call_site()) {
                return false;
            }
            return true;
        } else {
            return false;
        }
    }) {
        Ok(true)
    } else {
        Err(syn::parse::Error::new(Span::call_site(), "Field \'ir:u32\' is required!"))
    }
}

fn parse_int_attr(ast: &DeriveInput, name: &str) -> Result<u32, syn::parse::Error> {
    if let Some(attr) = ast.attrs.iter().find(|a| { a.path.segments.len() == 1 && a.path.segments[0].ident == name }) {
        if let syn::Meta::List(ref meta) = attr.parse_meta().unwrap() {
            if let syn::NestedMeta::Lit(syn::Lit::Int(ref lit)) = meta.nested[0] {
                lit.base10_parse::<u32>()
            } else {
                Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected as u32", name)))
            }
        } else {
            Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected as u32", name)))
        }
    } else {
        Err(syn::parse::Error::new(Span::call_site(), format!("attr \"{}\" missed", name)))
    }
}

