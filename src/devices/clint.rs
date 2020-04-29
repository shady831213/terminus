use std::num::Wrapping;
use terminus_spaceport::memory::prelude::*;
use terminus_spaceport::irq::IrqVec;
use terminus_macros::*;
use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

struct TimerInner {
    freq: usize,
    cnt: u64,
    irq_vecs: Vec<Rc<IrqVec>>,
    mtimecmps: Vec<u64>,
}

impl TimerInner {
    fn new(freq: usize) -> TimerInner {
        TimerInner {
            freq,
            cnt: 0,
            irq_vecs: vec![],
            mtimecmps: vec![],
        }
    }

    fn cnt_tick(&mut self, n: u64) {
        let cnt: Wrapping<u64> = Wrapping(self.cnt);
        self.cnt = (cnt + Wrapping(n)).0
    }

    fn alloc_irq(&mut self) -> Rc<IrqVec> {
        let irq_vec = Rc::new(IrqVec::new(2));
        irq_vec.set_enable_uncheck(0, true);
        irq_vec.set_enable_uncheck(1, true);
        self.irq_vecs.push(irq_vec.clone());
        self.mtimecmps.push(0);
        irq_vec
    }

    fn tick(&mut self, n: u64) {
        self.cnt_tick(n);
        for (irq_vec, mtimecmp) in self.irq_vecs.iter().zip(self.mtimecmps.iter()) {
            irq_vec.set_pending_uncheck(1, false);
            if self.cnt >= *mtimecmp {
                irq_vec.sender(1).unwrap().send().unwrap();
            }
        }
    }
}

pub struct Timer(RefCell<TimerInner>);

impl Timer {
    pub fn new(freq: usize) -> Timer {
        Timer(RefCell::new(TimerInner::new(freq)))
    }

    pub fn alloc_irq(&self) -> Rc<IrqVec> {
        self.0.borrow_mut().alloc_irq()
    }

    pub fn tick(&self, n: u64) {
        self.0.borrow_mut().tick(n)
    }

    pub fn freq(&self) -> usize {
        self.0.borrow().freq
    }

    fn inner(&self) -> Ref<'_, TimerInner> {
        self.0.borrow()
    }

    fn inner_mut(&self) -> RefMut<'_, TimerInner> {
        self.0.borrow_mut()
    }
}


const MSIP_BASE: u64 = 0x0;
const MSIP_SIZE: u64 = 4;
const MTIMECMP_BASE: u64 = 0x4000;
const MTMIECMP_SIZE: u64 = 8;
const MTIME_BASE: u64 = 0xbff8;
const MTIME_SIZE: u64 = 8;

#[derive_io(Bytes, U32, U64)]
pub struct Clint(Rc<Timer>);

impl Clint {
    pub fn new(timer: &Rc<Timer>) -> Clint {
        Clint(timer.clone())
    }
}

impl BytesAccess for Clint {
    fn write(&self, addr: &u64, data: &[u8]) {
        if data.len() == 4 {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(data);
            U32Access::write(self, addr, u32::from_le_bytes(bytes))
        } else if data.len() == 8 {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(data);
            U64Access::write(self, addr, u64::from_le_bytes(bytes))
        }
    }

    fn read(&self, addr: &u64, data: &mut [u8]) {
        if data.len() == 4 {
            data.copy_from_slice(&U32Access::read(self, addr).to_le_bytes())
        } else if data.len() == 8 {
            data.copy_from_slice(&U64Access::read(self, addr).to_le_bytes())
        }
    }
}

impl U32Access for Clint {
    fn write(&self, addr: &u64, data: u32) {
        assert!((*addr).trailing_zeros() > 1, format!("U32Access:unaligned addr:{:#x}", addr));
        let mut timer = self.0.inner_mut();
        if *addr >= MSIP_BASE && *addr + 4 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = ((*addr - MSIP_BASE) >> 2) as usize;
            timer.irq_vecs[offset].set_pending_uncheck(0, (data & 1) != 0);
            return;
        } else if *addr >= MTIMECMP_BASE && *addr + 4 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((*addr - MTIMECMP_BASE) >> 3) as usize;
            if (*addr).trailing_zeros() == 2 {
                timer.mtimecmps[offset].set_bit_range(63, 32, data)
            } else {
                timer.mtimecmps[offset].set_bit_range(31, 0, data)
            };
            timer.tick(0);
            return;
        } else if *addr >= MTIME_BASE && *addr + 4 <= MTIME_BASE + MTIME_SIZE {
            return if (*addr).trailing_zeros() == 2 {
                timer.cnt.set_bit_range(63, 32, data)
            } else {
                timer.cnt.set_bit_range(31, 0, data)
            };
        }

        panic!("clint:U32Access Invalid addr!".to_string());
    }

    fn read(&self, addr: &u64) -> u32 {
        assert!((*addr).trailing_zeros() > 1, format!("U32Access:unaligned addr:{:#x}", addr));
        let timer = self.0.inner();
        if *addr >= MSIP_BASE && *addr + 4 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = ((*addr - MSIP_BASE) >> 2) as usize;
            return timer.irq_vecs[offset].pending(0).unwrap() as u32;
        } else if *addr >= MTIMECMP_BASE && *addr + 4 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((*addr - MTIMECMP_BASE) >> 3) as usize;
            return if (*addr).trailing_zeros() == 2 {
                timer.mtimecmps[offset] >> 32
            } else {
                timer.mtimecmps[offset]
            } as u32;
        } else if *addr >= MTIME_BASE && *addr + 4 <= MTIME_BASE + MTIME_SIZE {
            return if (*addr).trailing_zeros() == 2 {
                timer.cnt >> 32
            } else {
                timer.cnt
            } as u32;
        }

        panic!("clint:U32Access Invalid addr!".to_string());
    }
}


impl U64Access for Clint {
    fn write(&self, addr: &u64, data: u64) {
        assert!((*addr).trailing_zeros() > 2, format!("U64Access:unaligned addr:{:#x}", addr));

        let mut timer = self.0.inner_mut();
        if *addr >= MSIP_BASE && *addr + 8 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = (((*addr - MSIP_BASE) >> 3) << 1) as usize;
            timer.irq_vecs[offset].set_pending_uncheck(0, (data & 1) != 0);
            timer.irq_vecs[offset + 1].set_pending_uncheck(0, ((data >> 32) & 1) != 0);
            return;
        } else if *addr >= MTIMECMP_BASE && *addr + 8 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((*addr - MTIMECMP_BASE) >> 3) as usize;
            timer.mtimecmps[offset] = data;
            timer.tick(0);
            return;
        } else if *addr >= MTIME_BASE && *addr + 8 <= MTIME_BASE + MTIME_SIZE {
            return timer.cnt = data;
        }

        panic!("clint:U64Access Invalid addr!".to_string());
    }

    fn read(&self, addr: &u64) -> u64 {
        assert!((*addr).trailing_zeros() > 2, format!("U64Access:unaligned addr:{:#x}", addr));

        let timer = self.0.inner();
        if *addr >= MSIP_BASE && *addr + 8 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = (((*addr - MSIP_BASE) >> 3) << 1) as usize;
            return (timer.irq_vecs[offset].pending(0).unwrap() as u64) | ((timer.irq_vecs[offset + 1].pending(0).unwrap() as u64) << 32);
        } else if *addr >= MTIMECMP_BASE && *addr + 8 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            return timer.mtimecmps[offset];
        } else if *addr >= MTIME_BASE && *addr + 8 <= MTIME_BASE + MTIME_SIZE {
            return timer.cnt;
        }

        panic!("clint:U64Access Invalid addr!".to_string());
    }
}
