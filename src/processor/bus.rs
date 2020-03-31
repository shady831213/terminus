use terminus_global::*;
use terminus_spaceport::space::Space;
use terminus_spaceport::memory::region::{U32Access, U64Access, U16Access, U8Access, IOAccess, BytesAccess, Region};
use terminus_spaceport::derive_io;
use terminus_spaceport::memory::region;
use std::sync::Arc;
use std::ops::Deref;

#[derive_io(U8, U16, U32, U64)]
pub struct ProcessorBus {
    space: Arc<Space>
}

impl ProcessorBus {
    pub fn new(space: &Arc<Space>) -> ProcessorBus {
        ProcessorBus { space: space.clone() }
    }

    fn get_region(&self, addr: u64) -> region::Result<Arc<Region>> {
        if let Some(region) = self.space.get_region_by_addr(addr) {
            Ok(region)
        } else {
            Err(region::Error::AccessErr(addr, format!("invalid addr:{:#x}", addr)))
        }
    }
}

impl U8Access for ProcessorBus {
    fn write(&self, addr: u64, data: u8) -> region::Result<()> {
        let region = self.get_region(addr)?;
        U8Access::write(region.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u8> {
        let region = self.get_region(addr)?;
        U8Access::read(region.deref(), addr)
    }
}

impl U16Access for ProcessorBus {
    fn write(&self, addr: u64, data: u16) -> region::Result<()> {
        let region = self.get_region(addr)?;
        U16Access::write(region.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u16> {
        let region = self.get_region(addr)?;
        U16Access::read(region.deref(), addr)
    }
}

impl U32Access for ProcessorBus {
    fn write(&self, addr: u64, data: u32) -> region::Result<()> {
        let region = self.get_region(addr)?;
        U32Access::write(region.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u32> {
        let region = self.get_region(addr)?;
        U32Access::read(region.deref(), addr)
    }
}

impl U64Access for ProcessorBus {
    fn write(&self, addr: u64, data: u64) -> region::Result<()> {
        let region = self.get_region(addr)?;
        U64Access::write(region.deref(), addr, data)
    }

    fn read(&self, addr: u64) -> region::Result<u64> {
        let region = self.get_region(addr)?;
        U64Access::read(region.deref(), addr)
    }
}

