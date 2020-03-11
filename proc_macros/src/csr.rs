use syn::parse::{Parse, ParseStream, Result, Error};
use syn::{braced, Ident, Token, parse2};
use syn::punctuated::Punctuated;
use proc_macro2::Span;
use std::convert::TryInto;
use proc_macro2::TokenStream;


mod kw {
    syn::custom_keyword!(fields);
    syn::custom_keyword!(map32);
    syn::custom_keyword!(map64);
    syn::custom_keyword!(map);
}

#[derive(Debug)]
struct Csr {
    name: Ident,
    attrs: Punctuated<CsrAttr, Token![,]>,
}


impl Parse for Csr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        Ok(Csr {
            name: name,
            attrs: content.parse_terminated(CsrAttr::parse)?,
        })
    }
}

#[derive(Debug)]
struct Attr<K, T, P> {
    key: K,
    attrs: Punctuated<T, P>,
}

impl<K, T, P> Attr<K, T, P> {
    fn new(key: K, attrs: Punctuated<T, P>) -> Attr<K, T, P> {
        Attr { key, attrs }
    }
}


#[derive(Debug)]
enum CsrAttr {
    Fields(Attr<kw::fields, Field, Token![,]>),
    Map32(Attr<kw::map32, FieldMap, Token![,]>),
    Map64(Attr<kw::map64, FieldMap, Token![,]>),
    Map(Attr<kw::map, FieldMap, Token![,]>),
}

macro_rules! parse_attr {
    ( $stream: ident, $key: path, $rt: path, $child: ty) => {
        || {
            let span = $stream.span();
            $stream.parse::<$key>()?;
            let content;
            syn::braced !(content in $ stream);
            Ok($rt(Attr::new($key(span), content.parse_terminated( <$child>::parse)?)))
        }
    };
}

impl Parse for CsrAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::fields) {
            parse_attr!(input, kw::fields, CsrAttr::Fields, Field)()
        } else if lookahead.peek(kw::map) {
            parse_attr!(input, kw::map, CsrAttr::Map, FieldMap)()
        } else if lookahead.peek(kw::map32) {
            parse_attr!(input, kw::map32, CsrAttr::Map32, FieldMap)()
        } else if lookahead.peek(kw::map64) {
            parse_attr!(input, kw::map64, CsrAttr::Map64, FieldMap)()
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
enum FieldPrivilege {
    RO,
    WO,
    WR,
}

#[derive(Debug)]
struct Field {
    name: Ident,
    width: usize,
    privilege: FieldPrivilege,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Field {
            name: Ident::new("field", Span::call_site()),
            width: 1,
            privilege: FieldPrivilege::WR,
        })
    }
}

#[derive(Debug)]
struct FieldMap {
    field: Ident,
    pos: usize,
}

impl Parse for FieldMap {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FieldMap {
            field: Ident::new("field", Span::call_site()),
            pos: 1,
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
    let maps = expand_call!(get_attr!(csr.attrs, CsrAttr::Map)());
    let map32s = expand_call!(get_attr!(csr.attrs, CsrAttr::Map32)());
    let map64s = expand_call!(get_attr!(csr.attrs, CsrAttr::Map64)());

    quote! {struct A;}
}




