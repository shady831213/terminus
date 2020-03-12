use syn::parse::{Parse, ParseStream, Result, Error, ParseBuffer};
use syn::{parenthesized, braced, Ident, Token, parse2, LitInt};
use syn::punctuated::Punctuated;
use proc_macro2::Span;
use std::convert::TryInto;
use proc_macro2::TokenStream;
use std::collections::HashMap;
use terminus_global::{RegT, reg_len};

mod attr_kw {
    syn::custom_keyword!(fields);
    syn::custom_keyword!(fields32);
    syn::custom_keyword!(fields64);
}

#[derive(Debug)]
struct Csr {
    name: Ident,
    attrs: Punctuated<CsrAttr, Token![,]>,
}


impl Parse for Csr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content:ParseBuffer;
        braced!(content in input);
        Ok(Csr {
            name: name,
            attrs: content.parse_terminated(CsrAttr::parse)?,
        })
    }
}

type AttrPunctuated = Punctuated<Field, Token![;]>;

#[derive(Debug)]
struct Attr<K> {
    key: K,
    attrs: AttrPunctuated,
}

impl<K> Attr<K> {
    fn new(key: K, attrs: AttrPunctuated) -> Attr<K> {
        Attr { key, attrs }
    }
}


#[derive(Debug)]
enum CsrAttr {
    Fields(Attr<attr_kw::fields>),
    Fields32(Attr<attr_kw::fields32>),
    Fields64(Attr<attr_kw::fields64>),
}

macro_rules! parse_attr {
    ( $stream: ident, $key: path, $rt: path) => {
        || {
            let span = $stream.span();
            $stream.parse::<$key>()?;
            let content;
            syn::braced !(content in $ stream);
            Ok($rt(Attr::new($key(span), content.parse_terminated( <Field>::parse)?)))
        }
    };
}

impl Parse for CsrAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attr_kw::fields) {
            parse_attr!(input, attr_kw::fields, CsrAttr::Fields)()
        } else if lookahead.peek(attr_kw::fields32) {
            parse_attr!(input, attr_kw::fields32, CsrAttr::Fields32)()
        } else if lookahead.peek(attr_kw::fields64) {
            parse_attr!(input, attr_kw::fields64, CsrAttr::Fields64)()
        } else {
            Err(lookahead.error())
        }
    }
}

mod field_kw {
    syn::custom_keyword!(RO);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(RW);
}

#[derive(Debug)]
enum FieldPrivilege {
    RO(field_kw::RO),
    WO(field_kw::WO),
    RW(field_kw::RW),
}

#[derive(Debug)]
struct Field {
    name: Ident,
    msb: LitInt,
    lsb: LitInt,
    privilege: FieldPrivilege,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        use FieldPrivilege::*;
        let content:ParseBuffer;
        parenthesized!(content in input);
        let privilege = if content.peek(field_kw::RO) {
            content.parse::<field_kw::RO>()?;
            RO(field_kw::RO(content.span()))
        } else if content.peek(field_kw::WO) {
            content.parse::<field_kw::WO>()?;
            WO(field_kw::WO(content.span()))
        } else if content.peek(field_kw::RW) {
            content.parse::<field_kw::RW>()?;
            RW(field_kw::RW(content.span()))
        } else {
            return Err(Error::new(content.span(), "expect [RO|WR|WO]"));
        };
        input.parse::<Token![:]>()?;

        let msb: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let lsb: LitInt = input.parse()?;

        if msb.base10_parse::<usize>()? < lsb.base10_parse::<usize>()? {
            return Err(Error::new(msb.span(), format!("msb {} is smaller than lsb {} !", msb.to_string(), lsb.to_string())));
        }

        Ok(Field {
            name,
            msb,
            lsb,
            privilege,
        })
    }
}

macro_rules! get_attr {
    ($attrs: expr, $exp: path) => {
        || {
            let _attr = $attrs.iter().filter_map(|f| {
                if let $exp(a) = f {
                    Some(a)
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            if _attr.len() == 0 {
                Ok(None)
            } else if _attr.len() == 1 {
                Ok(Some(_attr[0]))
            } else {
                Err(Error::new(_attr[1].key.span, format!("{:?} is redefined!", _attr[1].key)))
            }

        }
    };
}

macro_rules! expand_call {
    ($exp:expr) => {
        match $exp {
            Ok(result) => result,
            Err(err) => return err.to_compile_error(),
        }
    };
}


pub fn expand(input: TokenStream) -> TokenStream {
    let csr: Csr = expand_call!(syn::parse2(input));
    let fields = expand_call!(get_attr!(csr.attrs, CsrAttr::Fields)());
    let fields32 = expand_call!(get_attr!(csr.attrs, CsrAttr::Fields32)());
    let fields64 = expand_call!(get_attr!(csr.attrs, CsrAttr::Fields64)());

    // //build fields table
    // let mut fields_table: HashMap<String, &Field> = HashMap::new();
    // if let Some(Attr { key, attrs }) = fields {
    //     for field in attrs {
    //         if fields_table.insert(field.name.to_string(), field).is_some() {
    //             return Error::new(field.name.span(), format!("field {} is redefined!", field.name.to_string())).to_compile_error();
    //         }
    //     }
    // }
    //
    // //build map32 & map64 table
    // let mut map32s_table: HashMap<String, &FieldMap> = HashMap::new();
    // let mut map64s_table: HashMap<String, &FieldMap> = HashMap::new();
    // if let Some(Attr { key, attrs }) = maps {
    //     for map in attrs {
    //         if fields_table.get(&map.name.to_string()).is_none() {
    //             return Error::new(map.name.span(), format!("field name {} is invalid!", map.name.to_string())).to_compile_error();
    //         }
    //         if map32s_table.insert(map.name.to_string(), map).is_some() {
    //             return Error::new(map.name.span(), format!("map {} is redefined!", map.name.to_string())).to_compile_error();
    //         }
    //         if map64s_table.insert(map.name.to_string(), map).is_some() {
    //             return Error::new(map.name.span(), format!("map {} is redefined!", map.name.to_string())).to_compile_error();
    //         }
    //     }
    // }
    // if let Some(Attr { key, attrs }) = map32s {
    //     for map in attrs {
    //         if fields_table.get(&map.name.to_string()).is_none() {
    //             return Error::new(map.name.span(), format!("field name {} is invalid!", map.name.to_string())).to_compile_error();
    //         }
    //         if map32s_table.insert(map.name.to_string(), map).is_some() {
    //             return Error::new(map.name.span(), format!("map {} is redefined!", map.name.to_string())).to_compile_error();
    //         }
    //     }
    // }
    // if let Some(Attr { key, attrs }) = map64s {
    //     for map in attrs {
    //         if fields_table.get(&map.name.to_string()).is_none() {
    //             return Error::new(map.name.span(), format!("field name {} is invalid!", map.name.to_string())).to_compile_error();
    //         }
    //         if map64s_table.insert(map.name.to_string(), map).is_some() {
    //             return Error::new(map.name.span(), format!("map {} is redefined!", map.name.to_string())).to_compile_error();
    //         }
    //     }
    // }
    //
    // //default fields and maps
    // if fields_table.is_empty() {
    //     let ident = Ident::new(&csr.name.to_string().to_lowercase(), csr.name.span());
    //     fields_table.insert(ident.to_string(),
    //                         &Field {
    //                             name: ident.clone(),
    //                             width: LitInt::new(&format!("{}", reg_len()), ident.span()),
    //                             privilege: FieldPrivilege::RW(field_kw::RW(ident.span())),
    //                         });
    //
    //     let map = FieldMap { name: ident.clone(), pos: LitInt::new("0", ident.span()) };
    //     map32s_table.insert(ident.to_string(), &map);
    //     map64s_table.insert(ident.to_string(), &map);
    // }


    println!("{:?}", fields);
    println!("{:?}", fields32);
    println!("{:?}", fields64);

    quote! {struct A;}
}




