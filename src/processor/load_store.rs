use crate::devices::bus::CoreBus;
use crate::prelude::RegT;
use crate::processor::mmu::{Mmu, MmuOpt};
use crate::processor::trap::Exception;
use crate::processor::ProcessorState;
use std::rc::Rc;

pub struct LoadStore {
    bus: Rc<dyn CoreBus>,
}

impl LoadStore {
    pub fn new<B:CoreBus+'static>(bus: &Rc<B>) -> LoadStore {
        LoadStore { bus: bus.clone() }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    pub fn load_byte(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        data: &mut u8,
        mmu: &Mmu,
    ) -> Result<(), Exception> {
        let pa = mmu.ls_translate(state, addr, 1, MmuOpt::Load)?;
        match self.bus.read_u8(&pa, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::LoadAccess(*addr)),
        }
    }

    pub fn load_half_word(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        data: &mut u16,
        mmu: &Mmu,
    ) -> Result<(), Exception> {
        if addr.trailing_zeros() < 1 {
            return Err(Exception::LoadMisaligned(*addr));
        }
        let pa = mmu.ls_translate(state, addr, 2, MmuOpt::Load)?;
        match self.bus.read_u16(&pa, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::LoadAccess(*addr)),
        }
    }
    pub fn load_word(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        data: &mut u32,
        mmu: &Mmu,
    ) -> Result<(), Exception> {
        if addr.trailing_zeros() < 2 {
            return Err(Exception::LoadMisaligned(*addr));
        }
        let pa = mmu.ls_translate(state, addr, 4, MmuOpt::Load)?;
        match self.bus.read_u32(&pa, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::LoadAccess(*addr)),
        }
    }
    pub fn load_double_word(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        data: &mut u64,
        mmu: &Mmu,
    ) -> Result<(), Exception> {
        if addr.trailing_zeros() < 3 {
            return Err(Exception::LoadMisaligned(*addr));
        }
        let pa = mmu.ls_translate(state, addr, 8, MmuOpt::Load)?;
        match self.bus.read_u64(&pa, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::LoadAccess(*addr)),
        }
    }
    pub fn store_byte(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        data: &u8,
        mmu: &Mmu,
    ) -> Result<(), Exception> {
        let pa = mmu.ls_translate(state, addr, 1, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 1) {
            if lock_holder != state.hartid {
                self.bus.invalid_lock(addr, 1, lock_holder);
            }
        }
        match self.bus.write_u8(&pa, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::StoreAccess(*addr)),
        }
    }
    pub fn store_half_word(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        data: &u16,
        mmu: &Mmu,
    ) -> Result<(), Exception> {
        if addr.trailing_zeros() < 1 {
            return Err(Exception::StoreMisaligned(*addr));
        }
        let pa = mmu.ls_translate(state, addr, 2, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 2) {
            if lock_holder != state.hartid {
                self.bus.invalid_lock(addr, 2, lock_holder);
            }
        }
        match self.bus.write_u16(&pa, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::StoreAccess(*addr)),
        }
    }
    pub fn store_word(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        data: &u32,
        mmu: &Mmu,
    ) -> Result<(), Exception> {
        if addr.trailing_zeros() < 2 {
            return Err(Exception::StoreMisaligned(*addr));
        }
        let pa = mmu.ls_translate(state, addr, 4, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 4) {
            if lock_holder != state.hartid {
                self.bus.invalid_lock(addr, 4, lock_holder);
            }
        }
        match self.bus.write_u32(&pa, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::StoreAccess(*addr)),
        }
    }
    pub fn store_double_word(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        data: &u64,
        mmu: &Mmu,
    ) -> Result<(), Exception> {
        if addr.trailing_zeros() < 3 {
            return Err(Exception::StoreMisaligned(*addr));
        }
        let pa = mmu.ls_translate(state, addr, 8, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 8) {
            if lock_holder != state.hartid {
                self.bus.invalid_lock(addr, 8, lock_holder);
            }
        }
        match self.bus.write_u64(&pa, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::StoreAccess(*addr)),
        }
    }

    fn amo_u32<F: Fn(u32) -> u32>(&self, addr: &u64, f: F) -> Result<u32, u64> {
        let mut read: u32 = 0;
        self.bus.read_u32(addr, &mut read)?;
        self.bus.write_u32(addr, &f(read))?;
        Ok(read)
    }
    fn amo_u64<F: Fn(u64) -> u64>(&self, addr: &u64, f: F) -> Result<u64, u64> {
        let mut read: u64 = 0;
        self.bus.read_u64(addr, &mut read)?;
        self.bus.write_u64(addr, &f(read))?;
        Ok(read)
    }

    pub fn amo_word<F: Fn(u32) -> u32>(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        f: F,
        mmu: &Mmu,
    ) -> Result<RegT, Exception> {
        if addr.trailing_zeros() < 2 {
            return Err(Exception::StoreMisaligned(*addr));
        }
        let pa = mmu.ls_translate(state, addr, 4, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 4) {
            if lock_holder != state.hartid {
                self.bus.invalid_lock(addr, 4, lock_holder);
            }
        }
        match self.amo_u32(&pa, f) {
            Ok(data) => Ok(data as RegT),
            Err(_) => Err(Exception::StoreAccess(*addr)),
        }
    }
    pub fn amo_double_word<F: Fn(u64) -> u64>(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        f: F,
        mmu: &Mmu,
    ) -> Result<RegT, Exception> {
        if addr.trailing_zeros() < 3 {
            return Err(Exception::StoreMisaligned(*addr));
        }
        let pa = mmu.ls_translate(state, addr, 8, MmuOpt::Store)?;
        if let Some(lock_holder) = self.bus.lock_holder(addr, 8) {
            if lock_holder != state.hartid {
                self.bus.invalid_lock(addr, 8, lock_holder);
            }
        }
        match self.amo_u64(&pa, f) {
            Ok(data) => Ok(data as RegT),
            Err(_) => Err(Exception::StoreAccess(*addr)),
        }
    }

    pub fn acquire(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        len: usize,
        mmu: &Mmu,
    ) -> Result<bool, Exception> {
        let pa = mmu.ls_translate(state, addr, len, MmuOpt::Load)?;
        Ok(self.bus.acquire(&pa, len, state.hartid))
    }

    pub fn check_lock(
        &self,
        state: &ProcessorState,
        addr: &RegT,
        len: usize,
        mmu: &Mmu,
    ) -> Result<bool, Exception> {
        let pa = mmu.ls_translate(state, addr, len, MmuOpt::Store)?;
        if let Some(holder) = self.bus.lock_holder(&pa, len) {
            if holder == state.hartid {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub fn release(&self, state: &ProcessorState) {
        self.bus.release(state.hartid)
    }
}
