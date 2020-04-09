use std::num::Wrapping;
use terminus_spaceport::derive_io;
use terminus_spaceport::memory::region::{BytesAccess, U8Access, U16Access, U32Access, U64Access, IOAccess};
use std::sync::Mutex;
use terminus_spaceport::irq::IrqVec;
use terminus_spaceport::memory::region;
use terminus_macros::*;

pub struct Timer {
    freq: usize,
    cnt: u64,
}

impl Timer {
    fn new(freq: usize) -> Timer {
        Timer {
            freq,
            cnt: 0,
        }
    }

    fn tick(&mut self, n: u64) {
        let cnt: Wrapping<u64> = Wrapping(self.cnt);
        self.cnt = (cnt + Wrapping(n)).0
    }
}

const MSIP_BASE: u64 = 0x0;
const MSIP_SIZE: u64 = 4;
const MTIMECMP_BASE: u64 = 0x4000;
const MTMIECMP_SIZE: u64 = 8;
const MTIME_BASE: u64 = 0xbff8;
const MTIME_SIZE: u64 = 8;

struct ClintInner {
    timer: Timer,
    irq_vecs: Vec<IrqVec>,
    mtimecmps: Vec<u64>,
}

#[derive_io(U32, U64)]
pub struct Clint(Mutex<ClintInner>);

impl Clint {}

impl U32Access for Clint {
    fn write(&self, addr: u64, data: u32) -> region::Result<()> {
        if addr.trailing_zeros() < 2 {
            return Err(region::Error::Misaligned(addr));
        }
        let mut state = self.0.lock().unwrap();
        if addr >= MSIP_BASE && addr + 4 <= MSIP_BASE + state.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = ((addr - MSIP_BASE) >> 2) as usize;
            state.irq_vecs[offset].clr_pending(0).unwrap();
            if data & 1 == 1 {
                state.irq_vecs[offset].set_pending(0).unwrap();
            }
            return Ok(());
        } else if addr >= MTIMECMP_BASE && addr + 4 <= MTIMECMP_BASE + state.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            return Ok(if addr.trailing_zeros() == 2 {
                state.mtimecmps[offset].set_bit_range(63, 32, data)
            } else {
                state.mtimecmps[offset].set_bit_range(31, 0, data)
            })
        } else if addr >= MTIME_BASE && addr + 4 <= MTIME_BASE + MTIME_SIZE {
            return Ok(if addr.trailing_zeros() == 2 {
                state.timer.cnt.set_bit_range(63, 32, data)
            } else {
                state.timer.cnt.set_bit_range(31, 0, data)
            })
        }

        Err(region::Error::AccessErr(addr, "clint:U32Access Invalid addr!".to_string()))
    }

    fn read(&self, addr: u64) -> region::Result<u32> {
        if addr.trailing_zeros() < 2 {
            return Err(region::Error::Misaligned(addr));
        }
        let state = self.0.lock().unwrap();
        if addr >= MSIP_BASE && addr + 4 <= MSIP_BASE + state.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = ((addr - MSIP_BASE) >> 2) as usize;
            return Ok(state.irq_vecs[offset].pending(0).unwrap() as u32);
        } else if addr >= MTIMECMP_BASE && addr + 4 <= MTIMECMP_BASE + state.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            return Ok(if addr.trailing_zeros() == 2 {
                state.mtimecmps[offset] >> 32
            } else {
                state.mtimecmps[offset]
            } as u32)
        } else if addr >= MTIME_BASE && addr + 4 <= MTIME_BASE + MTIME_SIZE {
            return Ok(if addr.trailing_zeros() == 2 {
                state.timer.cnt >> 32
            } else {
                state.timer.cnt
            } as u32)
        }

        Err(region::Error::AccessErr(addr, "clint:U32Access Invalid addr!".to_string()))
    }
}


impl U64Access for Clint {
    fn write(&self, addr: u64, data: u64) -> region::Result<()> {
        if addr.trailing_zeros() < 3 {
            return Err(region::Error::Misaligned(addr));
        }
        let mut state = self.0.lock().unwrap();
        if addr >= MSIP_BASE && addr + 8 <= MSIP_BASE + state.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = (((addr - MSIP_BASE) >> 3) << 1) as usize;
            state.irq_vecs[offset].clr_pending(0).unwrap();
            if data & 1 == 1 {
                state.irq_vecs[offset].set_pending(0).unwrap();
            }
            state.irq_vecs[offset + 1].clr_pending(0).unwrap();
            if (data >> 32) & 1 == 1 {
                state.irq_vecs[offset + 1].set_pending(0).unwrap();
            }
            return Ok(());
        } else if addr >= MTIMECMP_BASE && addr + 8 <= MTIMECMP_BASE + state.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            return Ok(state.mtimecmps[offset] = data)
        } else if addr >= MTIME_BASE && addr + 8 <= MTIME_BASE + MTIME_SIZE {
            return Ok(state.timer.cnt = data)
        }

        Err(region::Error::AccessErr(addr, "clint:U64Access Invalid addr!".to_string()))
    }

    fn read(&self, addr: u64) -> region::Result<u64> {
        if addr.trailing_zeros() < 3 {
            return Err(region::Error::Misaligned(addr));
        }
        let state = self.0.lock().unwrap();
        if addr >= MSIP_BASE && addr + 8 <= MSIP_BASE + state.irq_vecs.len() as u64 * MSIP_SIZE {
            let offset = (((addr - MSIP_BASE) >> 3) << 1) as usize;
            return Ok((state.irq_vecs[offset].pending(0).unwrap() as u64) | ((state.irq_vecs[offset + 1].pending(0).unwrap() as u64) << 32));
        } else if addr >= MTIMECMP_BASE && addr + 8 <= MTIMECMP_BASE + state.mtimecmps.len() as u64 * MTMIECMP_SIZE {
            let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
            return Ok(state.mtimecmps[offset])
        } else if addr >= MTIME_BASE && addr + 8 <= MTIME_BASE + MTIME_SIZE {
            return Ok(state.timer.cnt)
        }

        Err(region::Error::AccessErr(addr, "clint:U64Access Invalid addr!".to_string()))
    }
}