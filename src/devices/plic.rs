use std::rc::Rc;
use terminus_spaceport::irq::{IrqVec, IrqVecSender};
use terminus_spaceport::memory::prelude::*;
use std::cell::RefCell;
use std::borrow::Borrow;
use std::ops::Deref;

struct PlicInner {
    irq_vecs: Vec<IrqVecSender>,
    priority_threshold: Vec<u32>,
    irq_src: IrqVec,
    priority: Vec<u32>,
    enables: Vec<Vec<u32>>,
    num_src: usize,
}

impl PlicInner {
    fn new(max_len: usize) -> PlicInner {
        PlicInner {
            irq_vecs: vec![],
            priority_threshold: vec![],
            irq_src: IrqVec::new(max_len),
            priority: vec![0; max_len],
            enables: vec![],
            num_src: 1,
        }
    }

    fn alloc_irq(&mut self) -> IrqVec {
        let irq_vec = IrqVec::new(1);
        let len = self.priority.len();
        irq_vec.set_enable_uncheck(0, true);
        self.irq_vecs.push(irq_vec.sender(0).unwrap());
        self.priority_threshold.push(0);
        self.enables.push(vec![0; len]);
        irq_vec
    }

    fn alloc_src(&mut self, id: usize) -> IrqVecSender {
        assert!(id != 0);
        self.num_src += 1;
        self.irq_src.enable(id).unwrap();
        self.irq_src.sender(id).unwrap()
    }

    fn update_irq(&self) {
        for vec in self.irq_vecs.iter() {
            vec.clear().unwrap();
        }
        for i in 1..self.num_src {
            if self.irq_src.pending_uncheck(i) {
                for (vec, (ths, enables)) in self.irq_vecs.iter().zip(self.priority_threshold.iter().zip(self.enables.iter())) {
                    if unsafe {
                        (*enables.get_unchecked(i >> 5) >> (i as u32 & 0x1f)) & 0x1 == 0x1 && *self.priority.get_unchecked(i) > *ths
                    } {
                        vec.send().unwrap();
                    }
                }
            }
        }
    }

    fn pick_claim(&self) -> u32 {
        let mut max_pri: u32 = 0;
        let mut idx: u32 = 0;
        for i in 1..self.num_src {
            if self.irq_src.pending_uncheck(i) {
                let pri = unsafe { self.priority.get_unchecked(i) };
                if *pri == 0x7 {
                    return i as u32;
                } else if *pri > max_pri {
                    idx = i as u32
                }
            }
        }
        idx
    }
}

const PLIC_PRI_BASE: u64 = 0x0;
const PLIC_PENDING_BASE: u64 = 0x1000;
const PLIC_ENABLE_BASE: u64 = 0x2000;
const PLIC_HART_BASE: u64 = 0x200000;
const PLIC_HART_PRI_TH: u64 = 0x0;
const PLIC_HART_CLAIM: u64 = 0x4;

// #[derive_io(Bytes, U32, U64)]
// pub struct Plic {
//     inner: RefCell<PlicInner>,
//
// }
//
// impl Plic {
//     pub fn new(len: usize) -> Plic {
//         Plic {
//             inner: RefCell::new(PlicInner::new(len))
//         }
//     }
//
//     pub fn alloc_irq(&self) -> Rc<IrqVec> {
//         self.inner.borrow_mut().alloc_irq()
//     }
//
//     pub fn alloc_src(&self, id: usize) -> IrqVecSender {
//         self.inner.borrow().alloc_src(id)
//     }
// }
//
// impl BytesAccess for Plic {
//     fn write(&self, addr: &u64, data: &[u8]) {
//         if data.len() == 4 {
//             let mut bytes = [0; 4];
//             bytes.copy_from_slice(data);
//             U32Access::write(self, addr, u32::from_le_bytes(bytes))
//         } else if data.len() == 8 {
//             let mut bytes = [0; 8];
//             bytes.copy_from_slice(data);
//             U64Access::write(self, addr, u64::from_le_bytes(bytes))
//         }
//     }
//
//     fn read(&self, addr: &u64, data: &mut [u8]) {
//         if data.len() == 4 {
//             data.copy_from_slice(&U32Access::read(self, addr).to_le_bytes())
//         } else if data.len() == 8 {
//             data.copy_from_slice(&U64Access::read(self, addr).to_le_bytes())
//         }
//     }
// }
//
// impl U32Access for Plic {
//     fn write(&self, addr: &u64, data: u32) {
//         assert!((*addr).trailing_zeros() > 1, format!("U32Access:unaligned addr:{:#x}", addr));
//         let inner = self.inner.borrow_mut();
//         if *addr >= PLIC_PRI_BASE && *addr + 4 <= PLIC_PRI_BASE + ((inner.priority.len() as u64) << 2) {
//             let offset = ((*addr - PLIC_PRI_BASE) >> 2) as usize;
//             *inner.priority[offset] = data as u16;
//             return;
//         } else if *addr >= MTIMECMP_BASE && *addr + 4 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
//             let offset = ((*addr - MTIMECMP_BASE) >> 3) as usize;
//             if (*addr).trailing_zeros() == 2 {
//                 timer.mtimecmps[offset].set_bit_range(63, 32, data)
//             } else {
//                 timer.mtimecmps[offset].set_bit_range(31, 0, data)
//             };
//             timer.tick(0);
//             return;
//         } else if *addr >= MTIME_BASE && *addr + 4 <= MTIME_BASE + MTIME_SIZE {
//             return if (*addr).trailing_zeros() == 2 {
//                 timer.cnt.set_bit_range(63, 32, data)
//             } else {
//                 timer.cnt.set_bit_range(31, 0, data)
//             };
//         }
//
//         panic!("clint:U32Access Invalid addr!".to_string());
//     }
//
//     fn read(&self, addr: &u64) -> u32 {
//         assert!((*addr).trailing_zeros() > 1, format!("U32Access:unaligned addr:{:#x}", addr));
//         let timer = self.0.inner();
//         if *addr >= MSIP_BASE && *addr + 4 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
//             let offset = ((*addr - MSIP_BASE) >> 2) as usize;
//             return timer.irq_vecs[offset].pending(0).unwrap() as u32;
//         } else if *addr >= MTIMECMP_BASE && *addr + 4 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
//             let offset = ((*addr - MTIMECMP_BASE) >> 3) as usize;
//             return if (*addr).trailing_zeros() == 2 {
//                 timer.mtimecmps[offset] >> 32
//             } else {
//                 timer.mtimecmps[offset]
//             } as u32;
//         } else if *addr >= MTIME_BASE && *addr + 4 <= MTIME_BASE + MTIME_SIZE {
//             return if (*addr).trailing_zeros() == 2 {
//                 timer.cnt >> 32
//             } else {
//                 timer.cnt
//             } as u32;
//         }
//
//         panic!("clint:U32Access Invalid addr!".to_string());
//     }
// }
//
//
// impl U64Access for Clint {
//     fn write(&self, addr: &u64, data: u64) {
//         assert!((*addr).trailing_zeros() > 2, format!("U64Access:unaligned addr:{:#x}", addr));
//
//         let mut timer = self.0.inner_mut();
//         if *addr >= MSIP_BASE && *addr + 8 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
//             let offset = (((*addr - MSIP_BASE) >> 3) << 1) as usize;
//             timer.irq_vecs[offset].set_pending_uncheck(0, (data & 1) != 0);
//             timer.irq_vecs[offset + 1].set_pending_uncheck(0, ((data >> 32) & 1) != 0);
//             return;
//         } else if *addr >= MTIMECMP_BASE && *addr + 8 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
//             let offset = ((*addr - MTIMECMP_BASE) >> 3) as usize;
//             timer.mtimecmps[offset] = data;
//             timer.tick(0);
//             return;
//         } else if *addr >= MTIME_BASE && *addr + 8 <= MTIME_BASE + MTIME_SIZE {
//             return timer.cnt = data;
//         }
//
//         panic!("clint:U64Access Invalid addr!".to_string());
//     }
//
//     fn read(&self, addr: &u64) -> u64 {
//         assert!((*addr).trailing_zeros() > 2, format!("U64Access:unaligned addr:{:#x}", addr));
//
//         let timer = self.0.inner();
//         if *addr >= MSIP_BASE && *addr + 8 <= MSIP_BASE + timer.irq_vecs.len() as u64 * MSIP_SIZE {
//             let offset = (((*addr - MSIP_BASE) >> 3) << 1) as usize;
//             return (timer.irq_vecs[offset].pending(0).unwrap() as u64) | ((timer.irq_vecs[offset + 1].pending(0).unwrap() as u64) << 32);
//         } else if *addr >= MTIMECMP_BASE && *addr + 8 <= MTIMECMP_BASE + timer.mtimecmps.len() as u64 * MTMIECMP_SIZE {
//             let offset = ((addr - MTIMECMP_BASE) >> 3) as usize;
//             return timer.mtimecmps[offset];
//         } else if *addr >= MTIME_BASE && *addr + 8 <= MTIME_BASE + MTIME_SIZE {
//             return timer.cnt;
//         }
//
//         panic!("clint:U64Access Invalid addr!".to_string());
//     }
// }


