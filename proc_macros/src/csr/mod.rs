use syn::parse::{Parse, ParseStream, Result, Error};
use proc_macro2::TokenStream;

macro_rules! expand_call {
    ($exp:expr) => {
        match $exp {
            Ok(result) => result,
            Err(err) => return err.to_compile_error(),
        }
    };
}

fn map_fold<T, O, I: Iterator<Item=T>, MF: Fn(T) -> O, FF: Fn(O, O) -> O>(iter: I, m: MF, init: O, f: FF) -> O {
    iter.map(m).fold(init, f)
}

fn quote_map_fold<T, I: Iterator<Item=T>, MF: Fn(T) -> TokenStream>(iter: I, m: MF) -> TokenStream {
    map_fold(iter, m, quote! {}, |acc, q| {
        quote! {
                    #acc
                    #q
                }
    })
}

mod privilege_kw {
    syn::custom_keyword!(RO);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(RW);
}

#[derive(Debug)]
enum CsrPrivilege {
    RO(privilege_kw::RO),
    WO(privilege_kw::WO),
    RW(privilege_kw::RW),
}

impl CsrPrivilege {
    fn writeable(&self) -> bool {
        match self {
            CsrPrivilege::RW(_) => true,
            CsrPrivilege::WO(_) => true,
            CsrPrivilege::RO(_) => false
        }
    }

    fn readable(&self) -> bool {
        match self {
            CsrPrivilege::RW(_) => true,
            CsrPrivilege::WO(_) => false,
            CsrPrivilege::RO(_) => true
        }
    }
}

impl Parse for CsrPrivilege {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(privilege_kw::RO) {
            input.parse::<privilege_kw::RO>()?;
            Ok(CsrPrivilege::RO(privilege_kw::RO(input.span())))
        } else if input.peek(privilege_kw::WO) {
            input.parse::<privilege_kw::WO>()?;
            Ok(CsrPrivilege::WO(privilege_kw::WO(input.span())))
        } else if input.peek(privilege_kw::RW) {
            input.parse::<privilege_kw::RW>()?;
            Ok(CsrPrivilege::RW(privilege_kw::RW(input.span())))
        } else {
            Err(Error::new(input.span(), "expect [RO|WR|WO]"))
        }
    }
}


pub mod define_csr;
pub mod csr_map;