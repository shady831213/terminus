use terminus_spaceport::memory::region::{U8Access, U16Access, U32Access, U64Access};
use terminus_spaceport::memory::region;
use crate::processor::ProcessorState;
use std::rc::Rc;
use terminus_global::RegT;
use crate::processor::mmu::{Mmu, MmuOpt};
use crate::processor::trap::Exception;

pub struct LoadStore {
    p: Rc<ProcessorState>,
}

impl LoadStore {
    pub fn new(p: &Rc<ProcessorState>) -> LoadStore {
        LoadStore {
            p: p.clone(),
        }
    }
    pub fn load_byte(&self, addr: RegT, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 1, MmuOpt::Load)?;
        match U8Access::read(&self.p.bus, pa) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::LoadAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::LoadMisaligned(addr))
            }
        }
    }
    pub fn load_half_word(&self, addr: RegT, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 2, MmuOpt::Load)?;
        match U16Access::read(&self.p.bus, pa) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::LoadAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::LoadMisaligned(addr))
            }
        }
    }
    pub fn load_word(&self, addr: RegT, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 4, MmuOpt::Load)?;
        match U32Access::read(&self.p.bus, pa) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::LoadAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::LoadMisaligned(addr))
            }
        }
    }
    pub fn load_double_word(&self, addr: RegT, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 8, MmuOpt::Load)?;
        match U64Access::read(&self.p.bus, pa) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::LoadAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::LoadMisaligned(addr))
            }
        }
    }
    pub fn store_byte(&self, addr: RegT, data: RegT, mmu: &Mmu) -> Result<(), Exception> {
        let pa = mmu.translate(addr, 1, MmuOpt::Store)?;
        match U8Access::write(&self.p.bus, pa, data as u8) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }
    pub fn store_half_word(&self, addr: RegT, data: RegT, mmu: &Mmu) -> Result<(), Exception> {
        let pa = mmu.translate(addr, 2, MmuOpt::Store)?;
        match U16Access::write(&self.p.bus, pa, data as u16) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }
    pub fn store_word(&self, addr: RegT, data: RegT, mmu: &Mmu) -> Result<(), Exception> {
        let pa = mmu.translate(addr, 4, MmuOpt::Store)?;
        match U32Access::write(&self.p.bus, pa, data as u32) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }
    pub fn store_double_word(&self, addr: RegT, data: RegT, mmu: &Mmu) -> Result<(), Exception> {
        let pa = mmu.translate(addr, 8, MmuOpt::Store)?;
        match U64Access::write(&self.p.bus, pa, data as u64) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }
}