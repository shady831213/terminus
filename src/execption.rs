use terminus_global::InsnT;
#[derive(Debug,Eq, PartialEq)]
pub enum Exception {
    IllegalInsn(InsnT),
    MemAccess(InsnT,u64),
    FetchAccess(u64),
    LoadAccess(u64),
    StoreAccess(u64),
    FetchPageFault(u64),
    LoadPageFault(u64),
    StorePageFault(u64),
}