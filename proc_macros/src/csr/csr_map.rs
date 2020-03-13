use syn::{parenthesized, braced, Ident, Token, LitInt, Path};
use syn::parse::{Parse, ParseStream, Result, Error, ParseBuffer};
use syn::punctuated::Punctuated;
use proc_macro2::TokenStream;
use super::*;

#[derive(Debug)]
struct CsrMaps {
    name: Ident,
    csrs: Punctuated<CsrMap, Token![;]>,
}

impl Parse for CsrMaps {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content: ParseBuffer;
        braced!(content in input);
        Ok(CsrMaps {
            name: name,
            csrs: content.parse_terminated(CsrMap::parse)?,
        })
    }
}

#[derive(Debug)]
struct CsrMap {
    name: Ident,
    privilege: CsrPrivilege,
    ty: Path,
    addr: LitInt,
}

impl Parse for CsrMap {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content: ParseBuffer;
        parenthesized!(content in input);
        let privilege = content.call(CsrPrivilege::parse)?;
        input.parse::<Token![:]>()?;

        let ty: Path = input.parse()?;
        input.parse::<Token![,]>()?;

        let addr: LitInt = input.parse()?;

        Ok(CsrMap {
            name,
            privilege,
            ty,
            addr,
        })
    }
}

pub fn expand(input: TokenStream) -> TokenStream {
    let maps: CsrMaps = expand_call!(syn::parse2(input));
    println!("{:?}", maps);
    quote! {}
}