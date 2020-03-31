use terminus_global::*;
use crate::execption::Exception;
use terminus_spaceport::space::Space;
use terminus_spaceport::memory::region::{U32Access, U64Access, U16Access, U8Access};
use std::sync::Arc;
use std::ops::Deref;

pub trait Bus {
    fn write8(&self, addr: u64, data: RegT) -> Result<(), Exception>;
    fn read8(&self, addr: u64) -> Result<RegT, Exception>;
    fn write16(&self, addr: u64, data: RegT) -> Result<(), Exception>;
    fn read16(&self, addr: u64) -> Result<RegT, Exception>;
    fn write32(&self, addr: u64, data: RegT) -> Result<(), Exception>;
    fn read32(&self, addr: u64) -> Result<RegT, Exception>;
    fn write64(&self, addr: u64, data: RegT) -> Result<(), Exception>;
    fn read64(&self, addr: u64) -> Result<RegT, Exception>;
}

pub struct MemoryBus {
    space: Arc<Space>
}

impl MemoryBus {
    pub fn new(space: &Arc<Space>) -> MemoryBus {
        MemoryBus { space: space.clone() }
    }
}

// impl Bus for MemoryBus {
//     fn write32(&self, addr: u64, data: RegT) -> Result<(), Exception> {
//         if let Some(region) = self.space.get_region_by_addr(addr) {
//             if addr.trailing_zero() < 2 {
//                 Err(Exception::BusMisalign(addr))
//             } else {
//                 U32Access::write(region.deref(), addr, data);
//                 Ok(())
//             }
//         } else {
//             Err(Exception::BusAccess(addr))
//         }
//     }
// }