use terminus_spaceport::memory::region::{U16Access, U32Access};
use terminus_spaceport::memory::region;
use crate::processor::ProcessorState;
use crate::processor::insn::Instruction;
use std::rc::Rc;
use terminus_global::{RegT, InsnT};
use crate::processor::mmu::{Mmu, MmuOpt};
use crate::processor::trap::Exception;
use crate::processor::decode::*;
use std::sync::Arc;
use crate::devices::bus::Bus;
use std::ops::Deref;
use std::cell::RefCell;
use std::mem::MaybeUninit;

struct ICacheEntry {
    accsessed: bool,
    tag: u64,
    insn: Option<Instruction>,
}

struct ICacheBasket {
    ptr: u8,
    entries: [ICacheEntry; 16],
}

impl ICacheBasket {
    fn new() -> ICacheBasket {
        ICacheBasket {
            ptr: 0,
            entries: unsafe {
                let mut arr: MaybeUninit<[ICacheEntry; 16]> = MaybeUninit::uninit();
                for i in 0..16 {
                    (arr.as_mut_ptr() as *mut ICacheEntry).add(i).write(ICacheEntry { accsessed: false, tag: 0, insn: None });
                }
                arr.assume_init()
            },
        }
    }

    fn get_insn(&mut self, tag: u64) -> Option<&Instruction> {
        let mut ptr = self.ptr;
        let tail = self.tail();
        while ptr != tail {
            if self.entries[ptr as usize].tag == tag {
                if let Some(ref i) = self.entries[ptr as usize].insn {
                    self.entries[ptr as usize].accsessed = true;
                    self.ptr = ptr;
                    return Some(i);
                }
            }
            self.entries[ptr as usize].accsessed = false;
            ptr = Self::next_ptr(ptr);
        }
        None
    }

    fn next_ptr(p: u8) -> u8 {
        if p == 15 {
            0
        } else {
            p + 1
        }
    }

    fn prev_ptr(p: u8) -> u8 {
        if p == 0 {
            15
        } else {
            p -1
        }
    }

    fn tail(&self) -> u8 {
        if self.ptr == 0 {
            15
        } else {
            self.ptr - 1
        }
    }

    fn set_entry(&mut self, tag: u64, insn: &Instruction) {
        let mut ptr = self.tail();
        let tail = self.ptr;
        while ptr != tail {
            let e = &self.entries[ptr as usize];
            if e.insn.is_none() || !e.accsessed {
                break;
            }
            ptr = Self::prev_ptr(ptr);
        }
        let e = &mut self.entries[ptr as usize];
        e.accsessed = true;
        e.tag = tag;
        e.insn = Some(insn.deref().deref().clone());
        self.ptr = ptr;
    }

    fn invalid_all(&mut self) {
        self.entries.iter_mut().for_each(|e| { e.insn = None })
    }
}


struct ICache {
    size: usize,
    baskets: Vec<ICacheBasket>,
}

impl ICache {
    fn new(size:usize) -> ICache {
        let mut cache = ICache {
            size,
            baskets:Vec::with_capacity(size),
        };
        for _ in 0 .. size {
            cache.baskets.push(ICacheBasket::new())
        };
        cache
    }

    fn get_insn(&mut self, addr: u64) -> Option<&Instruction> {
        self.baskets[((addr >> 1) as usize) & (self.size - 1)].get_insn(addr >> 1)
    }

    fn set_entry(&mut self, addr: u64, insn: &Instruction) {
        self.baskets[((addr >> 1) as usize) & (self.size - 1)].set_entry(addr >> 1, insn)
    }

    fn invalid_all(&mut self) {
        self.baskets.iter_mut().for_each(|b| { b.invalid_all() })
    }
}

pub struct Fetcher {
    p: Rc<ProcessorState>,
    bus: Arc<Bus>,
    icache: RefCell<ICache>,
}

impl Fetcher {
    pub fn new(p: &Rc<ProcessorState>, bus: &Arc<Bus>) -> Fetcher {
        Fetcher {
            p: p.clone(),
            bus: bus.clone(),
            icache: RefCell::new(ICache::new(1024)),
        }
    }
    #[inline(always)]
    fn fetch_u16_slow(&self, addr: u64, pc: u64) -> Result<InsnT, Exception> {
        match U16Access::read(self.bus.deref(), addr) {
            Ok(data) => {
                Ok(data as InsnT)
            }
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::FetchAccess(pc)),
                region::Error::Misaligned(_) => return Err(Exception::FetchMisaligned(pc))
            }
        }
    }
    #[inline(always)]
    fn fetch_u32_slow(&self, addr: u64, pc: u64) -> Result<InsnT, Exception> {
        match U32Access::read(self.bus.deref(), addr) {
            Ok(data) => Ok(data as InsnT),
            Err(e) => match e {
                region::Error::AccessErr(_, _) => return Err(Exception::FetchAccess(pc)),
                region::Error::Misaligned(_) => return Err(Exception::FetchMisaligned(pc))
            }
        }
    }

    pub fn flush_icache(&self) {
        self.icache.borrow_mut().invalid_all()
    }

    pub fn fetch(&self, pc: RegT, mmu: &Mmu) -> Result<Instruction, Exception> {
        let mut icache = self.icache.borrow_mut();
        if pc.trailing_zeros() == 1 {
            let pa = mmu.translate(pc, 2, MmuOpt::Fetch)?;
            if let Some(insn) = icache.get_insn(pa) {
                Ok(insn.deref().deref().clone())
            } else {
                let data_low = self.fetch_u16_slow(pa, pc)?;
                if data_low & 0x3 != 0x3 {
                    let data = data_low as u16 as InsnT;
                    let insn = GDECODER.decode(data)?;
                    icache.set_entry(pa, &insn);
                    Ok(insn)
                } else {
                    let pa_high = mmu.translate(pc + 2, 2, MmuOpt::Fetch)?;
                    let data_high = self.fetch_u16_slow(pa_high, pc)?;
                    let data = data_low as u16 as InsnT | ((data_high as u16 as InsnT) << 16);
                    let insn = GDECODER.decode(data)?;
                    icache.set_entry(pa, &insn);
                    Ok(insn)
                }
            }
        } else {
            let pa = mmu.translate(pc, 4, MmuOpt::Fetch)?;
            if let Some(insn) = icache.get_insn(pa) {
                Ok(insn.deref().deref().clone())
            } else {
                let data = self.fetch_u32_slow(pa, pc)?;
                if data & 0x3 != 0x3 {
                    let data_low = data as u16 as InsnT;
                    let insn = GDECODER.decode(data_low)?;
                    icache.set_entry(pa, &insn);
                    Ok(insn)
                } else {
                    let insn = GDECODER.decode(data)?;
                    icache.set_entry(pa, &insn);
                    Ok(insn)
                }
            }
        }

        // let code = {
        //     let mut icache = self.icache.borrow_mut();
        //     if pc.trailing_zeros() == 1 {
        //         let pa = mmu.translate(pc, 2, MmuOpt::Fetch)?;
        //         if let Some(data) = icache.get_insn(pa) {
        //             data
        //         } else {
        //             let data_low = self.fetch_u16_slow(pa, pc)?;
        //             if data_low & 0x3 != 0x3 {
        //                 let data = data_low as u16 as InsnT;
        //                 icache.set_entry(pa, data);
        //                 data
        //             } else {
        //                 let pa_high = mmu.translate(pc + 2, 2, MmuOpt::Fetch)?;
        //                 let data_high = self.fetch_u16_slow(pa_high, pc)?;
        //                 let data = data_low as u16 as InsnT | ((data_high as u16 as InsnT) << 16);
        //                 icache.set_entry(pa, data);
        //                 data
        //             }
        //         }
        //     } else {
        //         let pa = mmu.translate(pc, 4, MmuOpt::Fetch)?;
        //         if let Some(data) = icache.get_insn(pa) {
        //             data
        //         } else {
        //             let data = self.fetch_u32_slow(pa, pc)?;
        //             if data & 0x3 != 0x3 {
        //                 let data_low = data as u16 as InsnT;
        //                 icache.set_entry(pa, data_low);
        //                 data_low
        //             } else {
        //                 icache.set_entry(pa, data as InsnT);
        //                 data as InsnT
        //             }
        //         }
        //     }
        // };
        // GDECODER.decode(code)
    }
}