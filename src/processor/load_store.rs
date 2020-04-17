use terminus_spaceport::memory::region::{U8Access, U16Access, U32Access, U64Access};
use terminus_spaceport::memory::region;
use crate::processor::ProcessorState;
use std::rc::Rc;
use terminus_global::RegT;
use crate::processor::mmu::{Mmu, MmuOpt};
use crate::processor::trap::Exception;
use std::sync::Arc;
use crate::system::Bus;
use std::ops::Deref;

pub struct LoadStore {
    p: Rc<ProcessorState>,
    bus: Arc<Bus>,
}

impl LoadStore {
    pub fn new(p: &Rc<ProcessorState>, bus: &Arc<Bus>) -> LoadStore {
        LoadStore {
            p: p.clone(),
            bus: bus.clone(),
        }
    }
    pub fn reset(&self) {
        self.release()
    }

    pub fn load_byte(&self, addr: RegT, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 1, MmuOpt::Load)?;
        match U8Access::read(self.bus.deref(), pa) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::LoadAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::LoadMisaligned(addr))
            }
        }
    }
    pub fn load_half_word(&self, addr: RegT, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 2, MmuOpt::Load)?;
        match U16Access::read(self.bus.deref(), pa) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::LoadAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::LoadMisaligned(addr))
            }
        }
    }
    pub fn load_word(&self, addr: RegT, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 4, MmuOpt::Load)?;
        match U32Access::read(self.bus.deref(), pa) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::LoadAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::LoadMisaligned(addr))
            }
        }
    }
    pub fn load_double_word(&self, addr: RegT, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 8, MmuOpt::Load)?;
        match U64Access::read(self.bus.deref(), pa) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::LoadAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::LoadMisaligned(addr))
            }
        }
    }
    pub fn store_byte(&self, addr: RegT, data: RegT, mmu: &Mmu) -> Result<(), Exception> {
        let pa = mmu.translate(addr, 1, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 1) {
            if lock_holder != self.p.hartid {
                self.bus.invalid_lock(addr, 1, lock_holder);
            }
        }
        match U8Access::write(self.bus.deref(), pa, data as u8) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }
    pub fn store_half_word(&self, addr: RegT, data: RegT, mmu: &Mmu) -> Result<(), Exception> {
        let pa = mmu.translate(addr, 2, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 2) {
            if lock_holder != self.p.hartid {
                self.bus.invalid_lock(addr, 2, lock_holder);
            }
        }
        match U16Access::write(self.bus.deref(), pa, data as u16) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }
    pub fn store_word(&self, addr: RegT, data: RegT, mmu: &Mmu) -> Result<(), Exception> {
        let pa = mmu.translate(addr, 4, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 4) {
            if lock_holder != self.p.hartid {
                self.bus.invalid_lock(addr, 4, lock_holder);
            }
        }
        match U32Access::write(self.bus.deref(), pa, data as u32) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }
    pub fn store_double_word(&self, addr: RegT, data: RegT, mmu: &Mmu) -> Result<(), Exception> {
        let pa = mmu.translate(addr, 8, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 8) {
            if lock_holder != self.p.hartid {
                self.bus.invalid_lock(addr, 8, lock_holder);
            }
        }
        match U64Access::write(self.bus.deref(), pa, data as u64) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }

    pub fn amo_word<F: Fn(u32) -> u32>(&self, addr: RegT, f: F, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 4, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 4) {
            if lock_holder != self.p.hartid {
                self.bus.invalid_lock(addr, 4, lock_holder);
            }
        }
        match self.bus.amo_u32(pa, f) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }
    pub fn amo_double_word<F: Fn(u64) -> u64>(&self, addr: RegT, f: F, mmu: &Mmu) -> Result<RegT, Exception> {
        let pa = mmu.translate(addr, 8, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 8) {
            if lock_holder != self.p.hartid {
                self.bus.invalid_lock(addr, 8, lock_holder);
            }
        }
        match self.bus.amo_u64(pa, f) {
            Ok(data) => Ok(data as RegT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::StoreAccess(addr)),
                region::Error::Misaligned(_) => return Err(Exception::StoreMisaligned(addr))
            }
        }
    }

    pub fn acquire(&self, addr: RegT, len: u64, mmu: &Mmu) -> Result<bool, Exception> {
        let pa = mmu.translate(addr, len, MmuOpt::Load)?;
        Ok(self.bus.acquire(pa, len, self.p.hartid))
    }

    pub fn check_lock(&self, addr: RegT, len: u64, mmu: &Mmu) -> Result<bool, Exception> {
        let pa = mmu.translate(addr, len, MmuOpt::Store)?;
        if let Some(holder) = self.bus.lock_holder(pa, len) {
            if holder == self.p.hartid {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub fn release(&self) {
        self.bus.release(self.p.hartid)
    }
}