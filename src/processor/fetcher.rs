use crate::devices::bus::Bus;
use crate::prelude::*;
use crate::processor::mmu::Mmu;
use crate::processor::trap::Exception;
use crate::processor::ProcessorState;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;
struct ICacheEntry {
    accessed: bool,
    tag: u64,
    insn: Option<(InsnT, &'static Instruction)>,
}

struct ICacheBasket {
    ptr: u8,
    entries: [ICacheEntry; 4],
}

impl ICacheBasket {
    fn new() -> ICacheBasket {
        ICacheBasket {
            ptr: 0,
            entries: unsafe {
                let mut arr: MaybeUninit<[ICacheEntry; 4]> = MaybeUninit::uninit();
                for i in 0..4 {
                    (arr.as_mut_ptr() as *mut ICacheEntry)
                        .add(i)
                        .write(ICacheEntry {
                            accessed: false,
                            tag: 0,
                            insn: None,
                        });
                }
                arr.assume_init()
            },
        }
    }

    fn get_insn(&mut self, tag: u64) -> Option<&(InsnT, &'static Instruction)> {
        let mut ptr = self.ptr;
        let tail = self.tail();
        while ptr != tail {
            let e = unsafe { self.entries.get_unchecked_mut(ptr as usize) };
            if e.tag == tag {
                if e.insn.is_some() {
                    e.accessed = true;
                    self.ptr = ptr;
                    break;
                }
            }
            e.accessed = false;
            ptr = Self::next_ptr(ptr);
        }
        if ptr != tail {
            return unsafe { self.entries.get_unchecked(ptr as usize) }
                .insn
                .as_ref();
        }
        None
    }

    fn next_ptr(p: u8) -> u8 {
        if p == 3 {
            0
        } else {
            p + 1
        }
    }

    fn prev_ptr(p: u8) -> u8 {
        if p == 0 {
            3
        } else {
            p - 1
        }
    }

    fn tail(&self) -> u8 {
        if self.ptr == 0 {
            3
        } else {
            self.ptr - 1
        }
    }

    fn set_entry(&mut self, tag: u64, ir: InsnT, insn: &'static Instruction) {
        let mut ptr = self.tail();
        let tail = self.ptr;
        while ptr != tail {
            let e = unsafe { self.entries.get_unchecked(ptr as usize) };
            if e.insn.is_none() || !e.accessed {
                break;
            }
            ptr = Self::prev_ptr(ptr);
        }
        let e = unsafe { self.entries.get_unchecked_mut(ptr as usize) };
        e.accessed = true;
        e.tag = tag;
        e.insn = Some((ir, insn));
        self.ptr = ptr;
    }

    fn invalid_all(&mut self) {
        self.entries.iter_mut().for_each(|e| e.insn = None)
    }

    fn invalid_by_vpn(&mut self, vpn: u64) {
        self.entries.iter_mut().for_each(|e| {
            if vpn == e.tag >> 11 {
                e.insn = None
            }
        })
    }
}

struct ICache {
    size: usize,
    baskets: Vec<ICacheBasket>,
}

impl ICache {
    fn new(size: usize) -> ICache {
        let mut cache = ICache {
            size,
            baskets: Vec::with_capacity(size),
        };
        for _ in 0..size {
            cache.baskets.push(ICacheBasket::new())
        }
        cache
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn get_insn(&mut self, addr: u64) -> Option<&(InsnT, &'static Instruction)> {
        unsafe {
            self.baskets
                .get_unchecked_mut(((addr >> 1) as usize) & (self.size - 1))
        }
        .get_insn(addr >> 1)
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn set_entry(&mut self, addr: u64, ir: InsnT, insn: &'static Instruction) {
        unsafe {
            self.baskets
                .get_unchecked_mut(((addr >> 1) as usize) & (self.size - 1))
        }
        .set_entry(addr >> 1, ir, insn)
    }

    fn invalid_all(&mut self) {
        self.baskets.iter_mut().for_each(|b| b.invalid_all())
    }

    fn invalid_by_vpn(&mut self, vpn: u64) {
        self.baskets.iter_mut().for_each(|b| b.invalid_by_vpn(vpn))
    }
}

pub struct Fetcher {
    bus: Rc<dyn Bus>,
    icache: RefCell<ICache>,
}

impl Fetcher {
    pub fn new<B:Bus+'static>(bus: &Rc<B>) -> Fetcher {
        Fetcher {
            bus: bus.clone(),
            icache: RefCell::new(ICache::new(1024)),
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn fetch_u16_slow(&self, addr: &u64, pc: &u64, data: &mut u16) -> Result<(), Exception> {
        match self.bus.read_u16(addr, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::FetchAccess(*pc)),
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn fetch_u32_slow(&self, addr: &u64, pc: &u64, data: &mut u32) -> Result<(), Exception> {
        match self.bus.read_u32(addr, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(Exception::FetchAccess(*pc)),
        }
    }

    pub fn flush_icache(&self) {
        self.icache.borrow_mut().invalid_all()
    }

    pub fn flush_icache_by_vpn(&self, vpn: u64) {
        self.icache.borrow_mut().invalid_by_vpn(vpn)
    }

    pub fn fetch(
        &self,
        state: &ProcessorState,
        mmu: &Mmu,
    ) -> Result<(InsnT, &'static Instruction), Exception> {
        let mut icache = self.icache.borrow_mut();
        let pc = state.pc();
        if let Some(res) = icache.get_insn(*pc) {
            return Ok(*res);
        }
        if pc.trailing_zeros() == 1 {
            let pa = mmu.fetch_translate(state, pc, 2)?;
            let mut data_low = 0;
            self.fetch_u16_slow(&pa, pc, &mut data_low)?;
            if data_low & 0x3 != 0x3 {
                let data = data_low as u16 as InsnT;
                let insn = GDECODER.decode(&data)?;
                icache.set_entry(*pc, data, insn);
                Ok((data, insn))
            } else {
                let pa_high = if (*pc & 0xfff) == 0xffe {
                    mmu.fetch_translate(state, &(*pc + 2), 2)?
                } else {
                    pa + 2
                };
                let mut data_high = 0;
                self.fetch_u16_slow(&pa_high, pc, &mut data_high)?;
                let data = data_low as u16 as InsnT | ((data_high as u16 as InsnT) << 16);
                let insn = GDECODER.decode(&data)?;
                icache.set_entry(*pc, data, insn);
                Ok((data, insn))
            }
        } else {
            let pa = mmu.fetch_translate(state, pc, 4)?;
            let mut data = 0;
            self.fetch_u32_slow(&pa, pc, &mut data)?;
            if data & 0x3 != 0x3 {
                let data_low = data as u16 as InsnT;
                let insn = GDECODER.decode(&data_low)?;
                icache.set_entry(*pc, data_low, insn);
                Ok((data_low, insn))
            } else {
                let insn = GDECODER.decode(&data)?;
                icache.set_entry(*pc, data, insn);
                Ok((data, insn))
            }
        }
    }
}
