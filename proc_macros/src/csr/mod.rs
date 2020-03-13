use syn::parse::{Parse, ParseStream, Result, Error};

macro_rules! expand_call {
    ($exp:expr) => {
        match $exp {
            Ok(result) => result,
            Err(err) => return err.to_compile_error(),
        }
    };
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