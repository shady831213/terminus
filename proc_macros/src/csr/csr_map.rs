use syn::{parenthesized, braced, Ident, Token, LitInt, Path, Visibility};
use syn::parse::{Parse, ParseStream, Result, Error, ParseBuffer};
use syn::punctuated::Punctuated;
use proc_macro2::TokenStream;
use super::*;
use terminus_global::RegT;

#[derive(Debug)]
struct CsrMaps {
    name: Ident,
    vis: Visibility,
    low: LitInt,
    high: LitInt,
    csrs: Punctuated<CsrMap, Token![;]>,
}


impl Parse for CsrMaps {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        let range: ParseBuffer;
        parenthesized!(range in input);
        let low: LitInt = range.parse()?;
        range.parse::<Token![,]>()?;
        let high: LitInt = range.parse()?;
        if low.base10_parse::<RegT>()? > high.base10_parse::<RegT>()? {
            return Err(Error::new(range.span(), format!("low {} is bigger than high {} !", low.to_string(), high.to_string())));
        }
        let csrs: ParseBuffer;
        braced!(csrs in input);
        Ok(CsrMaps {
            name,
            vis,
            low,
            high,
            csrs: csrs.parse_terminated(CsrMap::parse)?,
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

impl CsrMap {
    fn same_name(&self, rhs: &Self) -> bool {
        self.name.to_string() == rhs.name.to_string()
    }

    fn addr_value(&self) -> RegT {
        self.addr.base10_parse().unwrap()
    }

    fn same_addr(&self, rhs: &Self) -> bool {
        self.addr_value() == rhs.addr_value()
    }
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
        addr.base10_parse::<RegT>()?;

        Ok(CsrMap {
            name,
            privilege,
            ty,
            addr,
        })
    }
}

struct Maps<'a> {
    low: RegT,
    high: RegT,
    maps: Vec<&'a CsrMap>,
}

impl<'a> Maps<'a> {
    fn new(low: RegT, high: RegT) -> Self {
        Maps {
            low,
            high,
            maps: vec![],
        }
    }

    fn out_of_range(&self, csr_map: &CsrMap) -> bool {
        csr_map.addr_value() > self.high || csr_map.addr_value() < self.low
    }

    fn add(&mut self, csr_map: &'a CsrMap) -> Result<()> {
        if self.out_of_range(csr_map) {
            Err(Error::new(csr_map.addr.span(), format!("addr of {}(0x{:x}) is out of range(0x{:x}, 0x{:x})!", csr_map.name.to_string(), csr_map.addr_value(), self.low, self.high)))
        } else {
            for prev in self.maps.iter() {
                if csr_map.same_name(prev) {
                    return Err(Error::new(csr_map.name.span(), format!("map name {} is redefined!", csr_map.name.to_string())));
                }
                if csr_map.same_addr(prev) {
                    return Err(Error::new(csr_map.addr.span(), format!("addr of {}(0x{:x}) is overlapped with {}!", csr_map.name.to_string(), csr_map.addr_value(), prev.name.to_string())));
                }
            }
            Ok(self.maps.push(csr_map))
        }
    }

    fn expand(&self, name: &Ident, vis: &Visibility) -> TokenStream {
        let fields = quote_map_fold(self.maps.iter(), |csr_map| {
            let name = &csr_map.name;
            let ty = &csr_map.ty;
            quote! {pub #name:#ty,}
        });
        let new_fn = quote_map_fold(self.maps.iter(), |csr_map| {
            let name = &csr_map.name;
            let ty = &csr_map.ty;
            quote! {#name:#ty::new(xlen),}
        });
        let write_matchs = quote_map_fold(self.maps.iter(), |csr_map| {
            let name = &csr_map.name;
            let addr = &csr_map.addr;
            let block = if csr_map.privilege.writeable() {
                quote! {Some(self.#name.set(value))}
            } else {
                quote! {Some(())}
            };
            quote! {
                #addr => #block,
            }
        });
        let read_matchs = quote_map_fold(self.maps.iter(), |csr_map| {
            let name = &csr_map.name;
            let addr = &csr_map.addr;
            let block = if csr_map.privilege.readable() {
                quote! {Some(self.#name.get())}
            } else {
                quote! {Some(0)}
            };
            quote! { #addr => #block,}
        });
        quote! {
            #vis struct #name {
                #fields
            }

            impl #name {
                pub fn new(xlen:XLen)->#name {
                    #name{
                    #new_fn
                    }
                }

                pub fn write(&mut self, addr:RegT, value:RegT)->Option<()> {
                    match addr {
                        #write_matchs
                        _ => None
                    }
                }

                pub fn read(&self, addr:RegT) -> Option<RegT> {
                    match addr {
                        #read_matchs
                        _ => None
                    }
                }
            }
        }
    }
}


pub fn expand(input: TokenStream) -> TokenStream {
    let maps: CsrMaps = expand_call!(syn::parse2(input));
    let mut csr_maps = Maps::new(maps.low.base10_parse().unwrap(), maps.high.base10_parse().unwrap());
    for csr_map in maps.csrs.iter() {
        expand_call!(csr_maps.add(csr_map));
    };
    csr_maps.expand(&maps.name, &maps.vis)
}