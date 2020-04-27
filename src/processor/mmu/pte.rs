use terminus_global::{XLen, RegT};
use crate::processor::extensions::s::csrs::*;
use crate::devices::bus::Bus;

pub const PTE_BARE: u8 = 0;
pub const PTE_SV32: u8 = 1;
pub const PTE_SV39: u8 = 8;
pub const PTE_SV48: u8 = 9;
// pub const PTE_SV57: u8 = 10;
// pub const PTE_SV64: u8 = 11;

pub struct PteInfo {
    pub mode: u8,
    pub level: usize,
    pub size_shift: usize,
    pub page_size_shift: usize,
}

impl PteInfo {
    #[cfg_attr(feature = "no-inline", inline(never))]
    pub fn new(satp: &Satp) -> PteInfo {
        match satp.xlen {
            XLen::X32 => PteInfo {
                mode: satp.mode() as u8,
                level: 2,
                size_shift: 2,
                page_size_shift: 12,
            },
            XLen::X64 => {
                let mode = satp.mode() as u8;
                let level = match mode {
                    PTE_SV39 => 3,
                    PTE_SV48 => 4,
                    PTE_BARE => 0,
                    _ => unreachable!()
                };
                PteInfo {
                    mode: mode,
                    level,
                    size_shift: 3,
                    page_size_shift: 12,
                }
            }
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct PteAttr(u8);

impl PteAttr {
    pub fn v(&self) -> u8 {
        self.0 & 0x1
    }

    pub fn r(&self) -> u8 {
        (self.0 >> 1) & 0x1
    }

    pub fn w(&self) -> u8 {
        (self.0 >> 2) & 0x1
    }

    pub fn x(&self) -> u8 {
        (self.0 >> 3) & 0x1
    }

    pub fn u(&self) -> u8 {
        (self.0 >> 4) & 0x1
    }

    // pub fn g(&self) -> u8 {
    //     (self.0 >> 5) & 0x1
    // }

    pub fn a(&self) -> u8 {
        (self.0 >> 6) & 0x1
    }

    pub fn set_a(&mut self, value: u8) {
        self.0 = ((value & 1) << 6) | self.0 & 0xbf
    }

    pub fn d(&self) -> u8 {
        (self.0 >> 7) & 0x1
    }

    pub fn set_d(&mut self, value: u8) {
        self.0 = ((value & 1) << 7) | self.0 & 0x7f
    }
}

impl From<u8> for PteAttr {
    fn from(v: u8) -> Self {
        PteAttr(v)
    }
}


pub struct Sv32Vaddr(RegT);

impl Sv32Vaddr {
    fn vpn(&self, level: usize) -> RegT {
        match level {
            0 => (self.0 >> 12) & 0x3ff,
            1 => (self.0 >> 22) & 0x3ff,
            _ => unreachable!()
        }
    }
    fn value(&self) -> RegT {
        self.0
    }
    fn vpn_all(&self) -> RegT {
        (self.0 >> 12) & 0xfffff
    }
    fn offset(&self) -> RegT {
        self.0 & 0xfff
    }
}

pub struct Sv32Paddr(RegT);

impl Sv32Paddr {
    fn set_ppn(&mut self, level: usize, ppn: RegT) {
        match level {
            0 => self.0 = self.0 & 0xffffffff_ffc00fff | (ppn & 0x3ff) << 12,
            1 => self.0 = self.0 & 0xfffffffc_003fffff | (ppn & 0xfff) << 22,
            _ => {}
        }
    }
    fn value(&self) -> RegT {
        self.0 & ((1 << 34) - 1)
    }
    // fn offset(&self) -> RegT {
    //     self.0 & 0xfff
    // }
}

pub struct Sv32Pte(RegT);

impl Sv32Pte {
    fn ppn(&self, level: usize) -> RegT {
        match level {
            0 => (self.0 >> 10) & 0x3ff,
            1 => (self.0 >> 20) & 0xfff,
            _ => unreachable!()
        }
    }
    fn ppn_all(&self) -> RegT {
        (self.0 >> 10) & 0x3fffff
    }
    fn value(&self) -> RegT {
        self.0
    }
    // fn rsw(&self) -> RegT {
    //     (self.0 >> 8) & 0x3
    // }
    fn attr(&self) -> PteAttr {
        PteAttr::from(self.0 as u8)
    }
    fn set_attr(&mut self, attr: &PteAttr) {
        self.0 = self.0 & 0xffffffff_fffffff0 | attr.0 as RegT
    }
}

pub struct Sv39Vaddr(RegT);

impl Sv39Vaddr {
    fn vpn(&self, level: usize) -> RegT {
        match level {
            0 => (self.0 >> 12) & 0x1ff,
            1 => (self.0 >> 21) & 0x1ff,
            2 => (self.0 >> 30) & 0x1ff,
            _ => unreachable!()
        }
    }
    fn value(&self) -> RegT {
        self.0
    }
    fn vpn_all(&self) -> RegT {
        (self.0 >> 12) & 0x7ffffff
    }
    fn offset(&self) -> RegT {
        self.0 & 0xfff
    }
}

pub struct Sv39Paddr(RegT);

impl Sv39Paddr {
    fn set_ppn(&mut self, level: usize, ppn: RegT) {
        match level {
            0 => self.0 = self.0 & 0xffffffff_ffe00fff | (ppn & 0x1ff) << 12,
            1 => self.0 = self.0 & 0xffffffff_c01fffff | (ppn & 0x1ff) << 21,
            2 => self.0 = self.0 & 0xff000000_3fffffff | (ppn & 0x3ffffff) << 30,
            _ => {}
        }
    }
    // fn offset(&self) -> RegT {
    //     self.0 & 0xfff
    // }
    fn value(&self) -> RegT {
        self.0 & ((1 << 56) - 1)
    }
}

pub struct Sv39Pte(RegT);

impl Sv39Pte {
    fn ppn(&self, level: usize) -> RegT {
        match level {
            0 => (self.0 >> 10) & 0x1ff,
            1 => (self.0 >> 19) & 0x1ff,
            2 => (self.0 >> 28) & 0x3ffffff,
            _ => unreachable!()
        }
    }
    fn ppn_all(&self) -> RegT {
        (self.0 >> 10) & 0xfff_ffffffff
    }
    fn value(&self) -> RegT {
        self.0
    }
    // fn rsw(&self) -> RegT {
    //     (self.0 >> 8) & 0x3
    // }
    fn attr(&self) -> PteAttr {
        PteAttr::from(self.0 as u8)
    }
    fn set_attr(&mut self, attr: &PteAttr) {
        self.0 = self.0 & 0xffffffff_fffffff0 | attr.0 as RegT
    }
}

pub struct Sv48Vaddr(RegT);

impl Sv48Vaddr {
    fn vpn(&self, level: usize) -> RegT {
        match level {
            0 => (self.0 >> 12) & 0x1ff,
            1 => (self.0 >> 21) & 0x1ff,
            2 => (self.0 >> 30) & 0x1ff,
            3 => (self.0 >> 39) & 0x1ff,
            _ => unreachable!()
        }
    }
    fn value(&self) -> RegT {
        self.0
    }
    fn vpn_all(&self) -> RegT {
        (self.0 >> 12) & 0xf_ffffffff
    }
    fn offset(&self) -> RegT {
        self.0 & 0xfff
    }
}

pub struct Sv48Paddr(RegT);

impl Sv48Paddr {
    fn set_ppn(&mut self, level: usize, ppn: RegT) {
        match level {
            0 => self.0 = self.0 & 0xffffffff_ffe00fff | (ppn & 0x1ff) << 12,
            1 => self.0 = self.0 & 0xffffffff_c01fffff | (ppn & 0x1ff) << 21,
            2 => self.0 = self.0 & 0xffffff80_3fffffff | (ppn & 0x1ff) << 30,
            3 => self.0 = self.0 & 0xff00007f_ffffffff | (ppn & 0x1ffff) << 39,
            _ => {}
        }
    }
    // fn offset(&self) -> RegT {
    //     self.0 & 0xfff
    // }
    fn value(&self) -> RegT {
        self.0 & ((1 << 56) - 1)
    }
}

pub struct Sv48Pte(RegT);

impl Sv48Pte {
    fn ppn(&self, level: usize) -> RegT {
        match level {
            0 => (self.0 >> 10) & 0x1ff,
            1 => (self.0 >> 19) & 0x1ff,
            2 => (self.0 >> 28) & 0x1ff,
            3 => (self.0 >> 37) & 0x1ffff,
            _ => unreachable!()
        }
    }
    fn ppn_all(&self) -> RegT {
        (self.0 >> 10) & 0xfff_ffffffff
    }
    fn value(&self) -> RegT {
        self.0
    }
    // fn rsw(&self) -> RegT {
    //     (self.0 >> 8) & 0x3
    // }
    fn attr(&self) -> PteAttr {
        PteAttr::from(self.0 as u8)
    }
    fn set_attr(&mut self, attr: &PteAttr) {
        self.0 = self.0 & 0xffffffff_fffffff0 | attr.0 as RegT
    }
}


macro_rules! pt_export {
    ($name:ident, $vis:vis $method:ident, $rt:ty, $($args:ident : $ty:ty),*) => {
        #[cfg_attr(feature = "no-inline", inline(never))]
       $vis fn $method(&self, $($args : $ty,)*) -> $rt {
            match self {
                $name::Sv32(addr) => addr.$method($($args),*),
                $name::Sv39(addr)  => addr.$method($($args),*),
                $name::Sv48(addr) => addr.$method($($args),*),
            }
        }
    };
    ($name:ident, $vis:vis $method:ident, $rt:ty) => {
        #[cfg_attr(feature = "no-inline", inline(never))]
        $vis fn $method(&self) -> $rt {
            match self {
                $name::Sv32(addr) => addr.$method(),
                $name::Sv39(addr)  => addr.$method(),
                $name::Sv48(addr) => addr.$method(),
            }
        }
    };
}

pub enum Vaddr {
    Sv32(Sv32Vaddr),
    Sv39(Sv39Vaddr),
    Sv48(Sv48Vaddr),
}

impl Vaddr {
    #[cfg_attr(feature = "no-inline", inline(never))]
    pub fn new(mode: u8, addr: RegT) -> Vaddr {
        match mode {
            PTE_SV32 => Vaddr::Sv32(Sv32Vaddr(addr)),
            PTE_SV39 => Vaddr::Sv39(Sv39Vaddr(addr)),
            PTE_SV48 => Vaddr::Sv48(Sv48Vaddr(addr)),
            _ => panic!(format!("unsupported PteMode {:?}", mode))
        }
    }
    pt_export!(Vaddr, pub offset, RegT);
    pt_export!(Vaddr, pub vpn, RegT, level:usize);
    pt_export!(Vaddr, pub vpn_all, RegT);
    pt_export!(Vaddr, pub value, RegT);
}

pub enum Paddr {
    Sv32(Sv32Paddr),
    Sv39(Sv39Paddr),
    Sv48(Sv48Paddr),
}

impl Paddr {
    pub fn new(vaddr: &Vaddr, pte: &Pte, info: &PteInfo, level: usize) -> Paddr {
        let mut pa = match vaddr {
            Vaddr::Sv32(addr) => Paddr::Sv32(Sv32Paddr(addr.vpn_all() << 12 | addr.offset())),
            Vaddr::Sv39(addr) => Paddr::Sv39(Sv39Paddr(addr.vpn_all() << 12 | addr.offset())),
            Vaddr::Sv48(addr) => Paddr::Sv48(Sv48Paddr(addr.vpn_all() << 12 | addr.offset()))
        };
        for i in level..info.level {
            pa.set_ppn(i, pte.ppn(i))
        }
        pa
    }
    pt_export!(Paddr, pub value, RegT);

    pub fn set_ppn(&mut self, level: usize, ppn: RegT) {
        match self {
            Paddr::Sv32(addr) => addr.set_ppn(level, ppn),
            Paddr::Sv39(addr) => addr.set_ppn(level, ppn),
            Paddr::Sv48(addr) => addr.set_ppn(level, ppn),
        }
    }
}

pub enum Pte {
    Sv32(Sv32Pte),
    Sv39(Sv39Pte),
    Sv48(Sv48Pte),
}

impl Pte {
    pub fn new(mode: u8, value: RegT) -> Pte {
        match mode {
            PTE_SV32 => Pte::Sv32(Sv32Pte(value)),
            PTE_SV39 => Pte::Sv39(Sv39Pte(value)),
            PTE_SV48 => Pte::Sv48(Sv48Pte(value)),
            _ => panic!(format!("unsupported PteMode {:?}", mode))
        }
    }
    pub fn load(info: &PteInfo, bus: &Bus, addr: &u64) -> Result<Pte, u64> {
        let value = match info.size_shift {
            2 => {
                let mut data:u32= 0;
                bus.read_u32(addr,&mut data)?;
                data as RegT
            }
            3 => {
                let mut data:u64= 0;
                bus.read_u64(addr,&mut data)?;
                data as RegT
            }
            _ => unreachable!()
        };
        Ok(Pte::new(info.mode, value))
    }

    pub fn store(&self, bus: &Bus, addr: &u64) -> Result<(), u64> {
        match self {
            Pte::Sv32(_) => bus.write_u32(addr, &(self.value() as u32)),
            _ => bus.write_u64(addr, &(self.value() as u64))
        }
    }

    pt_export!(Pte, pub ppn, RegT, level:usize);
    pt_export!(Pte, pub ppn_all, RegT);
    pt_export!(Pte, pub attr, PteAttr);
    pt_export!(Pte, pub value, RegT);

    pub fn set_attr(&mut self, attr: &PteAttr) {
        match self {
            Pte::Sv32(addr) => addr.set_attr(attr),
            Pte::Sv39(addr) => addr.set_attr(attr),
            Pte::Sv48(addr) => addr.set_attr(attr),
        }
    }
}