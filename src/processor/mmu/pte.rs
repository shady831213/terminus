use crate::prelude::*;
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

macro_rules! default_bitrange {
    ($name:ident) => {
        impl BitRange<RegT> for $name {
            fn bit_range(&self, msb: usize, lsb: usize) -> RegT {
                let width = msb - lsb + 1;
                if width == 64 {
                    self.0
                } else {
                    let mask: RegT = ((1 as RegT) << (width as RegT)) - 1;
                    (self.0 >> (lsb as RegT)) & mask
                }
            }

            fn set_bit_range(&mut self, msb: usize, lsb: usize, value: RegT) {
                let width = msb - lsb + 1;
                let bitlen = 64;
                if width == bitlen {
                    self.0 = value
                } else {
                    let low = self.0 & (((1 as RegT) << (lsb as RegT)) - 1);
                    let high = if msb == bitlen - 1 { 0 } else { (self.0 >> ((msb + 1) as RegT)) << ((msb + 1) as RegT) };
                    let mask: RegT = ((1 as RegT) << (width as RegT)) - 1;
                    self.0 = high | low | (((value as RegT) & mask) << (lsb as RegT));
                }
            }
        }
    };
}


bitfield! {
pub struct Sv32Vaddr(RegT);
no default BitRange;
impl Debug;
vpn1,_:31, 22;
vpn0,_:21, 12;
offset,_:11,0;
}

default_bitrange!(Sv32Vaddr);

impl Sv32Vaddr {
    fn vpn(&self, level: usize) -> Option<RegT> {
        match level {
            0 => Some(self.vpn0()),
            1 => Some(self.vpn1()),
            _ => None
        }
    }
    fn value(&self) -> RegT {
        self.0
    }
    fn vpn_all(&self) -> RegT {
        self.vpn1() << 10 | self.vpn0()
    }
}

bitfield! {
pub struct Sv32Paddr(RegT);
no default BitRange;
impl Debug;
ppn1,set_ppn1:33, 22;
ppn0,set_ppn0:21, 12;
offset,set_offset:11,0;
}

default_bitrange!(Sv32Paddr);

impl Sv32Paddr {
    // fn ppn(&self, level: usize) -> Option<RegT> {
    //     match level {
    //         0 => Some(self.ppn0()),
    //         1 => Some(self.ppn1()),
    //         _ => None
    //     }
    // }

    fn set_ppn(&mut self, level: usize, ppn: RegT) {
        match level {
            0 => self.set_ppn0(ppn),
            1 => self.set_ppn1(ppn),
            _ => {}
        }
    }
    fn value(&self) -> RegT {
        self.0 & ((1 << 32) - 1)
    }
}

bitfield! {
pub struct Sv32Pte(RegT);
no default BitRange;
impl Debug;
ppn1,_:31, 20;
ppn0,_:19, 10;
rsw,_:9,8;
attr_raw,set_attr_raw:7,0;
}
default_bitrange!(Sv32Pte);
impl Sv32Pte {
    fn ppn(&self, level: usize) -> Option<RegT> {
        match level {
            0 => Some(self.ppn0()),
            1 => Some(self.ppn1()),
            _ => None
        }
    }
    fn ppn_all(&self) -> RegT {
        self.ppn1() << 10 | self.ppn0()
    }
    fn value(&self) -> RegT {
        self.0
    }
}

bitfield! {
pub struct Sv39Vaddr(RegT);
no default BitRange;
impl Debug;
vpn2,_:38, 30;
vpn1,_:29, 21;
vpn0,_:20, 12;
offset,_:11,0;
}
default_bitrange!(Sv39Vaddr);

impl Sv39Vaddr {
    fn vpn(&self, level: usize) -> Option<RegT> {
        match level {
            0 => Some(self.vpn0()),
            1 => Some(self.vpn1()),
            2 => Some(self.vpn2()),
            _ => None
        }
    }
    fn value(&self) -> RegT {
        self.0
    }
    fn vpn_all(&self) -> RegT { self.vpn2() << 18 | self.vpn1() << 9 | self.vpn0() }
}

bitfield! {
pub struct Sv39Paddr(RegT);
no default BitRange;
impl Debug;
ppn2,set_ppn2:55, 30;
ppn1,set_ppn1:29, 21;
ppn0,set_ppn0:20, 12;
offset,set_offset:11,0;
}
default_bitrange!(Sv39Paddr);

impl Sv39Paddr {
    // fn ppn(&self, level: usize) -> Option<RegT> {
    //     match level {
    //         0 => Some(self.ppn0()),
    //         1 => Some(self.ppn1()),
    //         2 => Some(self.ppn2()),
    //         _ => None
    //     }
    // }

    fn set_ppn(&mut self, level: usize, ppn: RegT) {
        match level {
            0 => self.set_ppn0(ppn),
            1 => self.set_ppn1(ppn),
            2 => self.set_ppn2(ppn),
            _ => {}
        }
    }
    fn value(&self) -> RegT {
        self.0 & ((1 << 39) - 1)
    }
}

bitfield! {
pub struct Sv39Pte(RegT);
no default BitRange;
impl Debug;
ppn2,_:53, 28;
ppn1,_:27, 19;
ppn0,_:18, 10;
rsw,_:9,8;
attr_raw,set_attr_raw:7,0;
}
default_bitrange!(Sv39Pte);

impl Sv39Pte {
    fn ppn(&self, level: usize) -> Option<RegT> {
        match level {
            0 => Some(self.ppn0()),
            1 => Some(self.ppn1()),
            2 => Some(self.ppn2()),
            _ => None
        }
    }
    fn ppn_all(&self) -> RegT {
        self.ppn2() << 18 | self.ppn1() << 9 | self.ppn0()
    }
    fn value(&self) -> RegT {
        self.0
    }
}

bitfield! {
pub struct Sv48Vaddr(RegT);
no default BitRange;
impl Debug;
vpn3,_:47, 39;
vpn2,_:38, 30;
vpn1,_:29, 21;
vpn0,_:20, 12;
offset,_:11,0;
}
default_bitrange!(Sv48Vaddr);

impl Sv48Vaddr {
    fn vpn(&self, level: usize) -> Option<RegT> {
        match level {
            0 => Some(self.vpn0()),
            1 => Some(self.vpn1()),
            2 => Some(self.vpn2()),
            3 => Some(self.vpn3()),
            _ => None
        }
    }
    fn value(&self) -> RegT {
        self.0
    }
    fn vpn_all(&self) -> RegT { self.vpn3() << 27 | self.vpn2() << 18 | self.vpn1() << 9 | self.vpn0() }
}

bitfield! {
pub struct Sv48Paddr(RegT);
no default BitRange;
impl Debug;
ppn3,set_ppn3:55, 39;
ppn2,set_ppn2:38, 30;
ppn1,set_ppn1:29, 21;
ppn0,set_ppn0:20, 12;
offset,set_offset:11,0;
}
default_bitrange!(Sv48Paddr);

impl Sv48Paddr {
    // fn ppn(&self, level: usize) -> Option<RegT> {
    //     match level {
    //         0 => Some(self.ppn0()),
    //         1 => Some(self.ppn1()),
    //         2 => Some(self.ppn2()),
    //         3 => Some(self.ppn3()),
    //         _ => None
    //     }
    // }

    fn set_ppn(&mut self, level: usize, ppn: RegT) {
        match level {
            0 => self.set_ppn0(ppn),
            1 => self.set_ppn1(ppn),
            2 => self.set_ppn2(ppn),
            3 => self.set_ppn3(ppn),
            _ => {}
        }
    }
    fn value(&self) -> RegT {
        self.0 & ((1 << 48) - 1)
    }
}

bitfield! {
pub struct Sv48Pte(RegT);
no default BitRange;
impl Debug;
ppn3,_:53, 37;
ppn2,_:36, 28;
ppn1,_:27, 19;
ppn0,_:18, 10;
rsw,_:9,8;
attr_raw,set_attr_raw:7,0;
}
default_bitrange!(Sv48Pte);

impl Sv48Pte {
    fn ppn(&self, level: usize) -> Option<RegT> {
        match level {
            0 => Some(self.ppn0()),
            1 => Some(self.ppn1()),
            2 => Some(self.ppn2()),
            3 => Some(self.ppn3()),
            _ => None
        }
    }
    fn ppn_all(&self) -> RegT {
        self.ppn3() << 27 | self.ppn2() << 18 | self.ppn1() << 9 | self.ppn0()
    }
    fn value(&self) -> RegT {
        self.0
    }
}


macro_rules! pt_export {
    ($name:ident, $vis:vis $method:ident, $rt:ty, $($args:ident : $ty:ty),*) => {
       $vis fn $method(&self, $($args : $ty,)*) -> $rt {
            match self {
                $name::Sv32(addr) => addr.$method($($args),*),
                $name::Sv39(addr)  => addr.$method($($args),*),
                $name::Sv48(addr) => addr.$method($($args),*),
            }
        }
    };
    ($name:ident, $vis:vis $method:ident, $rt:ty) => {
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
    pub fn new(mode: u8, addr: RegT) -> Vaddr {
        match mode {
            PTE_SV32 => Vaddr::Sv32(Sv32Vaddr(addr)),
            PTE_SV39 => Vaddr::Sv39(Sv39Vaddr(addr)),
            PTE_SV48 => Vaddr::Sv48(Sv48Vaddr(addr)),
            _ => panic!(format!("unsupported PteMode {:?}", mode))
        }
    }
    pt_export!(Vaddr, pub offset, RegT);
    pt_export!(Vaddr, pub vpn, Option<RegT>, level:usize);
    pt_export!(Vaddr, pub vpn_all, RegT);
    pt_export!(Vaddr, pub value, RegT);
    // pt_export!(Vaddr, value, RegT);
}

pub enum Paddr {
    Sv32(Sv32Paddr),
    Sv39(Sv39Paddr),
    Sv48(Sv48Paddr),
}

impl Paddr {
    pub fn new(vaddr: &Vaddr, pte: &Pte, info: &PteInfo, level: usize) -> Paddr {
        let mut pa = match vaddr {
            Vaddr::Sv32(addr) => Paddr::Sv32(Sv32Paddr(addr.value())),
            Vaddr::Sv39(addr) => Paddr::Sv39(Sv39Paddr(addr.value())),
            Vaddr::Sv48(addr) => Paddr::Sv48(Sv48Paddr(addr.value()))
        };
        for i in level..info.level {
            pa.set_ppn(i, pte.ppn(i).unwrap())
        }
        pa
    }
    // pt_export!(Paddr, pub offset, RegT);
    // pt_export!(Paddr, pub ppn, Option<RegT>, level:usize);
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
    pub fn load(info: &PteInfo, bus: &Bus, addr: u64) -> Result<Pte, u64> {
        let value = match info.size_shift {
            2 => {
                bus.read_u32(addr)? as RegT
            }
            3 => {
                bus.read_u64(addr)? as RegT
            }
            _ => unreachable!()
        };
        Ok(Pte::new(info.mode, value))
    }

    pub fn store(&self, bus: &Bus, addr: u64) -> Result<(), u64> {
        match self {
            Pte::Sv32(_) => bus.write_u32(addr, self.value() as u32),
            _ => bus.write_u64(addr, self.value() as u64)
        }
    }

    // pt_export!(Pte, pub rsw, RegT);
    pt_export!(Pte, pub ppn, Option<RegT>, level:usize);
    pt_export!(Pte, pub ppn_all, RegT);
    pt_export!(Pte, attr_raw, RegT);
    pt_export!(Pte, pub value, RegT);
    pub fn attr(&self) -> PteAttr {
        (self.attr_raw() as u8).into()
    }

    pub fn set_attr(&mut self, attr: PteAttr) {
        match self {
            Pte::Sv32(addr) => addr.set_attr_raw(attr.0 as RegT),
            Pte::Sv39(addr) => addr.set_attr_raw(attr.0 as RegT),
            Pte::Sv48(addr) => addr.set_attr_raw(attr.0 as RegT),
        }
    }
}