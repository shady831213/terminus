extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::{DeriveInput, DataStruct, Ident};
use proc_macro2::Span;

fn insn_format_type() -> Vec<&'static str> {
    vec![
        "USER_DEFINE",
        "R",
        "I",
        "S",
        "B",
        "U",
        "J",
        "CR",
        "CIW",
        "CI",
        "CSS",
        "CL",
        "CS",
        "CB",
        "CJ",
    ]
}


#[proc_macro_derive(InsnCoding, attributes(code, mask, format))]
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
        let format = parse_format_attr(ast)?;
        check_fields(data)?;
        Ok(quote!(
            bitfield_bitrange!{struct #name(u32)}
            insn_format!(#name, #format);
            impl InsnCoding for #name {
                fn ir(&self) ->  u32 {
                    if (self.0 & self.mask() != self.code() & self.mask()) {
                        panic!(format!("maskd ir 0x{:x}, masked code 0x{:x} are not match! ir 0x{:x},  code 0x{:x},  mask 0x{:x}", self.0 & self.mask(), self.code() & self.mask(), self.0, self.code(), self.mask()))
                    }
                    self.0
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
    if let syn::Fields::Unnamed(ref field) =  data.fields {
        if field.unnamed.len() != 1 {
           return Err(syn::parse::Error::new(Span::call_site(), "expect struct \'name\' (u32)!"))
        }
        if let syn::Type::Path(ref path) = field.unnamed[0].ty {
            if path.path.segments.len() != 1 || path.path.segments[0].ident != Ident::new("u32", proc_macro2::Span::call_site()) {
                return Err(syn::parse::Error::new(Span::call_site(), "expect struct \'name\' (u32)!"))
            }
            return Ok(true)
        } else {
            return Err(syn::parse::Error::new(Span::call_site(), "expect struct \'name\' (u32)!"))
        }
    } else {
        Err(syn::parse::Error::new(Span::call_site(), "expect struct \'name\' (u32)!"))
    }
}

fn parse_int_attr(ast: &DeriveInput, name: &str) -> Result<u32, syn::parse::Error> {
    if let syn::NestedMeta::Lit(syn::Lit::Int(ref lit)) = parse_attr(ast, name)? {
        lit.base10_parse::<u32>()
    } else {
        Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected as u32", name)))
    }
}

fn parse_format_attr(ast: &DeriveInput) -> Result<Ident, syn::parse::Error> {
    if let syn::NestedMeta::Meta(syn::Meta::Path(ref path)) = parse_attr(ast, "format")? {
        if let Some(ident) = path.get_ident() {
            if insn_format_type().contains(&&format!("{}", ident)[..])  {
                Ok(ident.clone())
            } else {
                Err(syn::parse::Error::new(Span::call_site(), format!("invalid \"{}\" value \"{}\", valid values are {:?}", "format", ident, insn_format_type())))
            }
        } else {
            Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected as Ident", "format")))
        }
    } else {
        Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected as Ident", "format")))
    }
}

fn parse_attr(ast:&DeriveInput, name: &str) -> Result<syn::NestedMeta, syn::parse::Error> {
    if let Some(attr) = ast.attrs.iter().find(|a| { a.path.segments.len() == 1 && a.path.segments[0].ident == name }) {
        if let syn::Meta::List(ref meta) = attr.parse_meta().unwrap() {
            if meta.nested.len() == 1 {
                Ok(meta.nested[0].clone())
            } else {
                Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected to be a single value", name)))
            }
        } else {
            Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected to be a single value", name)))
        }
    } else {
        Err(syn::parse::Error::new(Span::call_site(), format!("attr \"{}\" missed", name)))
    }
}

