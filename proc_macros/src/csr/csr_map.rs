use syn::{parenthesized, braced, Ident, Token, LitInt, Path};
use syn::punctuated::Punctuated;
use super::*;

#[derive(Debug)]
struct CsrMap {
    name: Ident,
    csrs: Punctuated<Csr, Token![;]>,
}

#[derive(Debug)]
struct Csr {
    name: Ident,
    ty:Path,

}
