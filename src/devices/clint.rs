use std::num::Wrapping;
use terminus_spaceport::memory::prelude::*;
use std::sync::{Mutex, Arc, MutexGuard, LockResult};
use terminus_spaceport::irq::IrqVec;
use terminus_spaceport::memory::region;
use terminus_macros::*;

struct TimerInner {
    freq: usize,
    cnt: u64,
    irq_vecs: Vec<Arc<IrqVec>>,
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

    fn alloc_irq(&mut self) -> Arc<IrqVec> {
        let irq_vec = Arc::new(IrqVec::new(2));
        irq_vec.set_enable(0).unwrap();
        irq_vec.set_enable(1).unwrap();
        self.irq_vecs.push(irq_vec.clone());
        self.mtimecmps.push(0);
        irq_vec
    }

    fn tick(&mut self, n: u64) {
        self.cnt_tick(n);
        for (irq_vec, mtimecmp) in self.irq_vecs.iter().zip(self.mtimecmps.iter()) {
            irq_vec.clr_pending(1).unwrap();
            if self.cnt >= *mtimecmp {
                irq_vec.sender(1).unwrap().send().unwrap()
            }
        }
    }
}

pub struct Timer(Mutex<TimerInner>);

impl Timer {
    pub fn new(freq: usize) -> Timer {
        Timer(Mutex::new(TimerInner::new(freq)))
    }

    pub fn alloc_irq(&self) -> Arc<IrqVec> {
        self.0.lock().unwrap().alloc_irq()
    }

    pub fn tick(&self, n: u64) {
        self.0.lock().unwrap().tick(n)
    }

    pub fn freq(&self) -> usize {
        self.0.lock().unwrap().freq
    }

    fn lock(&self) -> LockResult<MutexGuard<'_, TimerInner>> {
        self.0.lock()
    }
}


const MSIP_BASE: u64 = 0x0;
const MSIP_SIZE: u64 = 4;
const MTIMECMP_BASE: u64 = 0x4000;
const MTMIECMP_SIZE: u64 = 8;
const MTIME_BASE: u64 = 0xbff8;
const MTIME_SIZE: u64 = 8;

#[derive_io(U32, U64)]
pub struct Clint(Arc<Timer>);

impl Clint {
    pub fn new(timer: &Arc<Timer>) -> Clint {
        Clint(timer.clone())
    }
}

impl U32Access for Clint {
    fn write(&self, addr: u64, data: u32) -> region::Result<()> {
        if addr.trailing_zeros() < 2 {
            return Err(region::Error::Misaligned(addr));
        }
        let mut timer = self.0.lock().unwrap();
        if addr >= MSIP_BASE && addr + 4 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = ((addr - MSIP_BASE) >> 2) as usize;
            timer.irq_vecs[offset].clr_pending(0).unwrap();
            if data & 1 == 1 {
                timer.irq_vecs[offset].set_pending(0).unwrap();
            }
            return Ok(());
        } else if addr >= MTIMECMP_BASE && addr + 4 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            timer.tick(0);
            return Ok(if addr.trailing_zeros() == 2 {
                timer.mtimecmps[offset].set_bit_range(63, 32, data)
            } else {
                timer.mtimecmps[offset].set_bit_range(31, 0, data)
            });
        } else if addr >= MTIME_BASE && addr + 4 <= MTIME_BASE + MTIME_SIZE {
            return Ok(if addr.trailing_zeros() == 2 {
                timer.cnt.set_bit_range(63, 32, data)
            } else {
                timer.cnt.set_bit_range(31, 0, data)
            });
        }

        Err(region::Error::AccessErr(addr, "clint:U32Access Invalid addr!".to_string()))
    }

    fn read(&self, addr: u64) -> region::Result<u32> {
        if addr.trailing_zeros() < 2 {
            return Err(region::Error::Misaligned(addr));
        }
        let timer = self.0.lock().unwrap();
        if addr >= MSIP_BASE && addr + 4 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = ((addr - MSIP_BASE) >> 2) as usize;
            return Ok(timer.irq_vecs[offset].pending(0).unwrap() as u32);
        } else if addr >= MTIMECMP_BASE && addr + 4 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            return Ok(if addr.trailing_zeros() == 2 {
                timer.mtimecmps[offset] >> 32
            } else {
                timer.mtimecmps[offset]
            } as u32);
        } else if addr >= MTIME_BASE && addr + 4 <= MTIME_BASE + MTIME_SIZE {
            return Ok(if addr.trailing_zeros() == 2 {
                timer.cnt >> 32
            } else {
                timer.cnt
            } as u32);
        }

        Err(region::Error::AccessErr(addr, "clint:U32Access Invalid addr!".to_string()))
    }
}


impl U64Access for Clint {
    fn write(&self, addr: u64, data: u64) -> region::Result<()> {
        if addr.trailing_zeros() < 3 {
            return Err(region::Error::Misaligned(addr));
        }
        let mut timer = self.0.lock().unwrap();
        if addr >= MSIP_BASE && addr + 8 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = (((addr - MSIP_BASE) >> 3) << 1) as usize;
            timer.irq_vecs[offset].clr_pending(0).unwrap();
            if data & 1 == 1 {
                timer.irq_vecs[offset].set_pending(0).unwrap();
            }
            timer.irq_vecs[offset + 1].clr_pending(0).unwrap();
            if (data >> 32) & 1 == 1 {
                timer.irq_vecs[offset + 1].set_pending(0).unwrap();
            }
            return Ok(());
        } else if addr >= MTIMECMP_BASE && addr + 8 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            timer.tick(0);
            return Ok(timer.mtimecmps[offset] = data);
        } else if addr >= MTIME_BASE && addr + 8 <= MTIME_BASE + MTIME_SIZE {
            return Ok(timer.cnt = data);
        }

        Err(region::Error::AccessErr(addr, "clint:U64Access Invalid addr!".to_string()))
    }

    fn read(&self, addr: u64) -> region::Result<u64> {
        if addr.trailing_zeros() < 3 {
            return Err(region::Error::Misaligned(addr));
        }
        let timer = self.0.lock().unwrap();
        if addr >= MSIP_BASE && addr + 8 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = (((addr - MSIP_BASE) >> 3) << 1) as usize;
            return Ok((timer.irq_vecs[offset].pending(0).unwrap() as u64) | ((timer.irq_vecs[offset + 1].pending(0).unwrap() as u64) << 32));
        } else if addr >= MTIMECMP_BASE && addr + 8 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            return Ok(timer.mtimecmps[offset]);
        } else if addr >= MTIME_BASE && addr + 8 <= MTIME_BASE + MTIME_SIZE {
            return Ok(timer.cnt);
        }

        Err(region::Error::AccessErr(addr, "clint:U64Access Invalid addr!".to_string()))
    }
}

#[cfg(test)]
use std::thread;
#[cfg(test)]
use std::time::Duration;

#[test]
fn timer_test() {
    let timer = Arc::new(Timer::new(100));
    let clint = Clint::new(&timer);
    let irq_vec = timer.alloc_irq();
    let p0 = thread::spawn({
        let irq = irq_vec.clone();
        move || {
            for cnt in 0..10 {
                while !irq.pending(1).unwrap() {}
                println!("get timer {}!", cnt);
                irq.clr_pending(1).unwrap();
                let time = U64Access::read(&clint, MTIME_BASE).unwrap();
                println!("time = {}", time);
                U64Access::write(&clint, MTIMECMP_BASE, time + 1).unwrap();
            }
        }
    }
    );

    thread::spawn({
        let t = timer.clone();
        move || {
            loop {
                thread::sleep(Duration::from_millis(5));
                t.tick(5);
            }
        }
    }
    );

    p0.join().unwrap();
}