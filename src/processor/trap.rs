use terminus_global::{InsnT, RegT};

#[derive(Debug, Copy, Clone)]
pub enum Trap {
    Exception(Exception),
    Interrupt(Interrupt),
}

impl From<Exception> for Trap {
    fn from(e: Exception) -> Self {
        Trap::Exception(e)
    }
}

impl From<Interrupt> for Trap {
    fn from(i: Interrupt) -> Self {
        Trap::Interrupt(i)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
    Breakpoint,
    UCall,
    SCall,
    MCall,
}

impl Exception {
    pub fn code(&self) -> RegT {
        match self {
            Exception::FetchMisaligned(_) => 0,
            Exception::FetchAccess(_) => 1,
            Exception::IllegalInsn(_) => 2,
            Exception::Breakpoint => 3,
            Exception::LoadMisaligned(_) => 4,
            Exception::LoadAccess(_) => 5,
            Exception::StoreMisaligned(_) => 6,
            Exception::StoreAccess(_) => 7,
            Exception::UCall => 8,
            Exception::SCall => 9,
            Exception::MCall => 11,
            Exception::FetchPageFault(_) => 12,
            Exception::LoadPageFault(_) => 13,
            Exception::StorePageFault(_) => 15,
        }
    }
    pub fn tval(&self) -> RegT {
        match self {
            Exception::FetchMisaligned(addr) => *addr as RegT,
            Exception::FetchAccess(addr) => *addr as RegT,
            Exception::IllegalInsn(inst) => *inst as RegT,
            Exception::Breakpoint => 0 as RegT,
            Exception::LoadMisaligned(addr) => *addr as RegT,
            Exception::LoadAccess(addr) => *addr as RegT,
            Exception::StoreMisaligned(addr) => *addr as RegT,
            Exception::StoreAccess(addr) => *addr as RegT,
            Exception::UCall => 0 as RegT,
            Exception::SCall => 0 as RegT,
            Exception::MCall => 0 as RegT,
            Exception::FetchPageFault(addr) => *addr as RegT,
            Exception::LoadPageFault(addr) => *addr as RegT,
            Exception::StorePageFault(addr) => *addr as RegT,
        }
    }

    pub fn executed(&self) -> bool {
        match self {
            Exception::Breakpoint => true,
            Exception::UCall => true,
            Exception::SCall => true,
            Exception::MCall => true,
            _ => false
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Interrupt {
    USInt,
    SSInt,
    MSInt,
    UTInt,
    STInt,
    MTInt,
    UEInt,
    SEInt,
    MEInt,
}

impl Interrupt {
    pub fn code(&self) -> RegT {
        match self {
            Interrupt::USInt => 0,
            Interrupt::SSInt => 1,
            Interrupt::MSInt => 3,
            Interrupt::UTInt => 4,
            Interrupt::STInt => 5,
            Interrupt::MTInt => 7,
            Interrupt::UEInt => 8,
            Interrupt::SEInt => 9,
            Interrupt::MEInt => 11,
        }
    }
    pub fn tval(&self) -> RegT {
        0
    }
}