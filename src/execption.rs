use terminus_global::InsnT;
#[derive(Debug,Eq, PartialEq)]
pub enum Exception {
    FetchMisaligned(u64),
    LoadMisaligned(u64),
    StoreMisaligned(u64),
    IllegalInsn(InsnT),
    FetchAccess(u64),
    LoadAccess(u64),
    StoreAccess(u64),
    FetchPageFault(u64),
    LoadPageFault(u64),
    StorePageFault(u64),
}