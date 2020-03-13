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


pub mod define_csr;
pub mod csr_map;