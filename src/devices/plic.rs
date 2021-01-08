use std::cell::{Ref, RefCell};
use std::rc::Rc;
use terminus_spaceport::irq::{IrqVec, IrqVecSender};
use terminus_spaceport::memory::prelude::*;

struct IntHarts {
    irq_vecs: Vec<IrqVecSender>,
    priority_threshold: Vec<u32>,
    priority: Vec<u32>,
    enables: Vec<Vec<u32>>,
}

impl IntHarts {
    fn new(max_len: usize) -> IntHarts {
        IntHarts {
            irq_vecs: vec![],
            priority_threshold: vec![],
            priority: vec![0; max_len],
            enables: vec![],
        }
    }
    fn alloc_irq(&mut self) -> IrqVec {
        let irq_vec = IrqVec::new(1);
        let len = self.priority.len();
        irq_vec.set_enable_uncheck(0, true);
        self.irq_vecs.push(irq_vec.sender(0).unwrap());
        self.priority_threshold.push(0);
        self.enables.push(vec![0; (len + 31) >> 5]);
        irq_vec
    }

    fn update_meip(&self, id: usize) {
        for (vec, (ths, enables)) in self
            .irq_vecs
            .iter()
            .zip(self.priority_threshold.iter().zip(self.enables.iter()))
        {
            if unsafe {
                (*enables.get_unchecked(id >> 5) >> (id as u32 & 0x1f)) & 0x1 == 0x1
                    && *self.priority.get_unchecked(id) > *ths
            } {
                vec.send().unwrap();
            }
        }
    }

    fn clear_all_meip(&self) {
        for vec in self.irq_vecs.iter() {
            vec.clear().unwrap();
        }
    }
}

struct IntcInner {
    harts: Rc<RefCell<IntHarts>>,
    irq_src: IrqVec,
    num_src: usize,
}

impl IntcInner {
    fn new(max_len: usize) -> IntcInner {
        IntcInner {
            harts: Rc::new(RefCell::new(IntHarts::new(max_len))),
            irq_src: IrqVec::new(max_len),
            num_src: 1,
        }
    }

    fn alloc_irq(&mut self) -> IrqVec {
        (*self.harts).borrow_mut().alloc_irq()
    }

    fn alloc_src(&mut self, id: usize) -> IrqVecSender {
        assert!(id != 0);
        self.num_src += 1;
        self.irq_src.set_enable_uncheck(id, true);
        self.irq_src
            .binder()
            .bind(id, {
                let harts = self.harts.clone();
                move || (*harts).borrow().update_meip(id)
            })
            .unwrap();
        self.irq_src.sender(id).unwrap()
    }

    fn update_all_meip(&self) {
        let harts = (*self.harts).borrow();
        harts.clear_all_meip();
        for i in 1..self.num_src {
            if self.irq_src.pending_uncheck(i) {
                harts.update_meip(i)
            }
        }
    }

    fn enable_per_hart(&self) -> usize {
        (self.num_src + 31) >> 5
    }

    fn pending(&self, offset: u64) -> u32 {
        let mut res: u32 = 0;
        let start = offset << 3;
        for i in start..start + 32 {
            res |= self.irq_src.pending_uncheck(i as usize) as u32
        }
        res
    }

    fn pick_claim(&self) -> u32 {
        let mut max_pri: u32 = 0;
        let mut idx: u32 = 0;
        let harts = (*self.harts).borrow();
        for i in 1..self.num_src {
            if self.irq_src.pending_uncheck(i) {
                let pri = unsafe { harts.priority.get_unchecked(i) };
                if *pri == 0x7 {
                    return i as u32;
                } else if *pri > max_pri {
                    max_pri = *pri;
                    idx = i as u32
                }
            }
        }
        idx
    }
}

pub struct Intc(RefCell<IntcInner>);

impl Intc {
    pub fn new(max_len: usize) -> Intc {
        Intc(RefCell::new(IntcInner::new(max_len)))
    }

    pub fn alloc_irq(&self) -> IrqVec {
        self.0.borrow_mut().alloc_irq()
    }

    pub fn alloc_src(&self, id: usize) -> IrqVecSender {
        self.0.borrow_mut().alloc_src(id)
    }

    pub fn num_src(&self) -> usize {
        self.0.borrow().num_src
    }

    fn inner(&self) -> Ref<'_, IntcInner> {
        self.0.borrow()
    }
}

const PLIC_PRI_BASE: u64 = 0x0;
const PLIC_PENDING_BASE: u64 = 0x1000;
const PLIC_ENABLE_BASE: u64 = 0x2000;
// const PLIC_ENABLE_SIZE:u64 = 0x80;
const PLIC_HART_BASE: u64 = 0x200000;
// const PLIC_HART_SIZE:u64 = 0x1000;

#[derive_io(Bytes, U32, U64)]
pub struct Plic(Rc<Intc>);

impl Plic {
    pub fn new(intc: &Rc<Intc>) -> Plic {
        Plic(intc.clone())
    }
}

impl BytesAccess for Plic {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        if data.len() == 4 {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(data);
            U32Access::write(self, addr, u32::from_le_bytes(bytes))
        } else if data.len() == 8 {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(data);
            U64Access::write(self, addr, u64::from_le_bytes(bytes))
        }
        Ok(0)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        if data.len() == 4 {
            data.copy_from_slice(&U32Access::read(self, addr).to_le_bytes())
        } else if data.len() == 8 {
            data.copy_from_slice(&U64Access::read(self, addr).to_le_bytes())
        }
        Ok(0)
    }
}

impl U32Access for Plic {
    fn write(&self, addr: &u64, data: u32) {
        assert!(
            (*addr).trailing_zeros() > 1,
            format!("U32Access:unaligned addr:{:#x}", addr)
        );
        let inner = self.0.inner();
        if *addr >= PLIC_PRI_BASE && *addr + 4 <= PLIC_PRI_BASE + ((inner.num_src as u64) << 2) {
            let offset = ((*addr - PLIC_PRI_BASE) >> 2) as usize;
            inner.harts.borrow_mut().priority[offset] = data & 0x7;
            return;
        } else if *addr >= PLIC_ENABLE_BASE
            && *addr + 4
                <= PLIC_ENABLE_BASE
                    + (((inner.num_src + 31) as u64) >> 3)
                        * (inner.harts.borrow().irq_vecs.len() as u64)
        {
            let offset = ((*addr - PLIC_ENABLE_BASE) >> 2) as usize;
            let enable_per_hart = inner.enable_per_hart();
            let hart_offset = offset / enable_per_hart;
            let en_offset = offset % enable_per_hart;
            inner.harts.borrow_mut().enables[hart_offset][en_offset] = data;
            return;
        } else if *addr >= PLIC_HART_BASE
            && *addr + 4 <= PLIC_HART_BASE + ((inner.harts.borrow().irq_vecs.len() as u64) << 3)
        {
            if (*addr).trailing_zeros() == 2 {
                inner.update_all_meip()
            } else {
                let offset = ((*addr - PLIC_HART_BASE) >> 3) as usize;
                inner.harts.borrow_mut().priority_threshold[offset] = data & 0x7;
            }
            return;
        }

        // panic!(format!("plic:U32Access Invalid addr {:#x}!", *addr));
    }

    fn read(&self, addr: &u64) -> u32 {
        assert!(
            (*addr).trailing_zeros() > 1,
            format!("U32Access:unaligned addr:{:#x}", addr)
        );
        let inner = self.0.inner();
        if *addr >= PLIC_PRI_BASE && *addr + 4 <= PLIC_PRI_BASE + ((inner.num_src as u64) << 2) {
            let offset = ((*addr - PLIC_PRI_BASE) >> 2) as usize;
            return inner.harts.borrow().priority[offset];
        } else if *addr >= PLIC_PENDING_BASE
            && *addr + 4 <= PLIC_PENDING_BASE + (((inner.num_src + 31) as u64) >> 3)
        {
            let offset = *addr - PLIC_PENDING_BASE;
            return inner.pending(offset);
        } else if *addr >= PLIC_ENABLE_BASE
            && *addr + 4
                <= PLIC_ENABLE_BASE
                    + (((inner.num_src + 31) as u64) >> 3)
                        * (inner.harts.borrow().irq_vecs.len() as u64)
        {
            let offset = ((*addr - PLIC_ENABLE_BASE) >> 2) as usize;
            let enable_per_hart = inner.enable_per_hart();
            let hart_offset = offset / enable_per_hart;
            let en_offset = offset % enable_per_hart;
            return inner.harts.borrow().enables[hart_offset][en_offset];
        } else if *addr >= PLIC_HART_BASE
            && *addr + 4 <= PLIC_HART_BASE + ((inner.harts.borrow().irq_vecs.len() as u64) << 3)
        {
            if (*addr).trailing_zeros() == 2 {
                let claim = inner.pick_claim();
                inner.irq_src.set_pending_uncheck(claim as usize, false);
                inner.update_all_meip();
                return claim;
            } else {
                let offset = ((*addr - PLIC_HART_BASE) >> 3) as usize;
                return inner.harts.borrow().priority_threshold[offset];
            }
        }
        0
        // panic!(format!("plic:U32Access Invalid addr {:#x}!", *addr));
    }
}

impl U64Access for Plic {
    fn write(&self, addr: &u64, data: u64) {
        assert!(
            (*addr).trailing_zeros() > 2,
            format!("U64Access:unaligned addr:{:#x}", addr)
        );
        U32Access::write(self, addr, data as u32);
        U32Access::write(self, &(*addr + 4), (data >> 32) as u32);
    }

    fn read(&self, addr: &u64) -> u64 {
        assert!(
            (*addr).trailing_zeros() > 2,
            format!("U64Access:unaligned addr:{:#x}", addr)
        );
        U32Access::read(self, addr) as u64 | ((U32Access::read(self, &(*addr + 4)) as u64) << 32)
    }
}
