use terminus_global::InsnT;
#[derive(Debug,Eq, PartialEq)]
pub enum Exception {
    IllegalInsn(InsnT),
    MemAccess(InsnT,u64),
    FetchAccess(InsnT, u64),
    LoadAccess(InsnT, u64),
    StoreAccess(InsnT, u64),
    BusAccess(u64),
    BusMisalign(u64),
}