use crate::processor::ProcessorState;
use crate::processor::insn::Instruction;
use terminus_global::InsnT;
use crate::processor::mmu::Mmu;
use crate::processor::trap::Exception;
use crate::processor::decode::*;
use std::sync::Arc;
use crate::devices::bus::Bus;
use std::cell::RefCell;

struct ICacheEntry {
    tag: u64,
    insn: Option<(InsnT, &'static Instruction)>,
}

struct ICache {
    size: usize,
    baskets: Vec<ICacheEntry>,
}

impl ICache {
    fn new(size: usize) -> ICache {
        let mut cache = ICache {
            size,
            baskets: Vec::with_capacity(size),
        };
        for _ in 0..size {
            cache.baskets.push(ICacheEntry { tag: 0, insn: None })
        };
        cache
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn get_insn(&mut self, addr: u64) -> Option<&(InsnT, &'static Instruction)> {
        let e = unsafe { self.baskets.get_unchecked(((addr >> 1) as usize) & (self.size - 1)) };
        if let Some(ref insn) = e.insn {
            if e.tag == addr >> 1 {
                Some(insn)
            } else {
                None
            }
        } else {
            None
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn set_entry(&mut self, addr: u64, ir: InsnT, insn: &'static Instruction) {
        let e = unsafe { self.baskets.get_unchecked_mut(((addr >> 1) as usize) & (self.size - 1)) };
        e.tag = addr >> 1;
        e.insn = Some((ir, insn));
    }

    fn invalid_all(&mut self) {
        self.baskets.iter_mut().for_each(|b| { b.insn = None })
    }
}

pub struct Fetcher {
    bus: Arc<Bus>,
    icache: RefCell<ICache>,
}

impl Fetcher {
    pub fn new(bus: &Arc<Bus>) -> Fetcher {
        Fetcher {
            bus: bus.clone(),
            icache: RefCell::new(ICache::new(1024)),
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn fetch_u16_slow(&self, addr: &u64, pc: &u64, data: &mut u16) -> Result<(), Exception> {
        match self.bus.read_u16(addr, data) {
            Ok(_) => {
                Ok(())
            }
            Err(_) => Err(Exception::FetchAccess(*pc)),
        }
    }
    #[cfg_attr(feature = "no-inline", inline(never))]
    fn fetch_u32_slow(&self, addr: &u64, pc: &u64, data: &mut u32) -> Result<(), Exception> {
        match self.bus.read_u32(addr, data) {
            Ok(_) => {
                Ok(())
            }
            Err(_) => Err(Exception::FetchAccess(*pc)),
        }
    }

    pub fn flush_icache(&self) {
        self.icache.borrow_mut().invalid_all()
    }

    pub fn fetch(&self, state: &ProcessorState, mmu: &Mmu) -> Result<(InsnT, &'static Instruction), Exception> {
        let mut icache = self.icache.borrow_mut();
        let pc = state.pc();
        if pc.trailing_zeros() == 1 {
            let pa = mmu.fetch_translate(state, pc, 2)?;
            if let Some(res) = icache.get_insn(pa) {
                Ok(*res)
            } else {
                let mut data_low = 0;
                self.fetch_u16_slow(&pa, pc, &mut data_low)?;
                if data_low & 0x3 != 0x3 {
                    let data = data_low as u16 as InsnT;
                    let insn = GDECODER.decode(data)?;
                    icache.set_entry(pa, data, insn);
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
                    let insn = GDECODER.decode(data)?;
                    icache.set_entry(pa, data, insn);
                    Ok((data, insn))
                }
            }
        } else {
            let pa = mmu.fetch_translate(state, pc, 4)?;
            if let Some(res) = icache.get_insn(pa) {
                Ok(*res)
            } else {
                let mut data = 0;
                self.fetch_u32_slow(&pa, pc, &mut data)?;
                if data & 0x3 != 0x3 {
                    let data_low = data as u16 as InsnT;
                    let insn = GDECODER.decode(data_low)?;
                    icache.set_entry(pa, data_low, insn);
                    Ok((data_low, insn))
                } else {
                    let insn = GDECODER.decode(data)?;
                    icache.set_entry(pa, data, insn);
                    Ok((data, insn))
                }
            }
        }
    }
}
