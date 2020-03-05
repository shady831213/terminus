extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate terminus_macros;

use proc_macro::TokenStream;
use syn::DeriveInput;
use proc_macro2::Span;
use terminus_macros::*;

#[proc_macro_derive(InsnCoding, attributes(code, mask))]
pub fn insn_coding(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    match insn_codeing_transform(&ast)  {
        Ok(s) => s.into(),
        Err(e) => e.to_compile_error().into()
    }
}

fn insn_codeing_transform(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, syn::parse::Error> {
    if let syn::Data::Struct(_) = &ast.data {
        let name = &ast.ident;
        let code = parse_int_attr(ast, "code")?;
        let mask =  parse_int_attr(ast, "mask")?;
        Ok(quote!(
            impl InsnCoding for #name {
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

fn parse_int_attr(ast:&DeriveInput, name:&str) -> Result<u32, syn::parse::Error> {
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

