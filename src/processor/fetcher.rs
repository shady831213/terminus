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
use std::collections::VecDeque;
use std::cell::RefCell;

#[derive(Clone)]
struct ICacheEntry {
    tag: u64,
    data: InsnT,
}

#[derive(Clone)]
struct ICacheBasket {
    size: usize,
    entries: VecDeque<ICacheEntry>,
}

impl ICacheBasket {
    fn new(size: usize) -> ICacheBasket {
        ICacheBasket {
            size,
            entries: VecDeque::new(),
        }
    }

    fn get_insn(&mut self, tag: u64) -> Option<InsnT> {
        let mut idx: Option<usize> = None;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.tag == tag {
                idx = Some(i);
                break;
            }
        }
        if let Some(i) = idx {
            if i != 0 {
                let entry = self.entries.remove(i).unwrap();
                self.entries.push_front(entry);
            }
            Some(self.entries[0].data)
        } else {
            None
        }
    }

    fn set_entry(&mut self, tag: u64, data: InsnT) {
        if self.entries.len() >= self.size {
            self.entries.pop_back();
        }
        self.entries.push_front(ICacheEntry { tag, data })
    }
}


struct ICache {
    size: usize,
    baskets: Vec<ICacheBasket>,
}

impl ICache {
    fn new(cache_size: usize, basket_size: usize) -> ICache {
        assert!(cache_size.is_power_of_two());
        ICache {
            size: cache_size,
            baskets: vec![ICacheBasket::new(basket_size); cache_size],
        }
    }

    fn get_insn(&mut self, addr: u64) -> Option<InsnT> {
        self.baskets[((addr >> 2) as usize) & (self.size - 1) ].get_insn(addr >> 1)
    }

    fn set_entry(&mut self, addr: u64, data: InsnT) {
        self.baskets[((addr >> 2) as usize) & (self.size - 1)].set_entry(addr >> 1, data)
    }

    fn invalid_all(&mut self) {
        let basket_size = self.baskets[0].size;
        self.baskets.iter_mut().for_each(|b|{*b = ICacheBasket::new(basket_size)})
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
            icache: RefCell::new(ICache::new(1024, 128)),
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
        let code = {
            let mut icache = self.icache.borrow_mut();
            if pc.trailing_zeros() == 1 {
                let pa = mmu.translate(pc, 2, MmuOpt::Fetch)?;
                if let Some(data) = icache.get_insn(pa) {
                    data
                } else {
                    let data_low = self.fetch_u16_slow(pa, pc)?;
                    if data_low & 0x3 != 0x3 {
                        let data = data_low as u16 as InsnT;
                        icache.set_entry(pa, data);
                        data
                    } else {
                        let pa_high = mmu.translate(pc + 2, 2, MmuOpt::Fetch)?;
                        let data_high = self.fetch_u16_slow(pa_high, pc)?;
                        let data = data_low as u16 as InsnT | ((data_high as u16 as InsnT) << 16);
                        icache.set_entry(pa, data);
                        data
                    }
                }
            } else {
                let pa = mmu.translate(pc, 4, MmuOpt::Fetch)?;
                if let Some(data) = icache.get_insn(pa) {
                    data
                } else {
                    let data = self.fetch_u32_slow(pa, pc)?;
                    if data & 0x3 != 0x3 {
                        let data_low = data as u16 as InsnT;
                        icache.set_entry(pa, data_low);
                        data_low
                    } else {
                        icache.set_entry(pa, data as InsnT);
                        data as InsnT
                    }
                }
            }
        };
        GDECODER.decode(code)
    }
}