use terminus_spaceport::memory::region;
use terminus_global::{XLen, RegT};
use std::convert::TryFrom;
use terminus_spaceport::memory::region::{U32Access, U64Access};
use crate::processor::extensions::i::csrs::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::system::Bus;

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PteMode {
    Bare = 0,
    Sv32 = 1,
    Sv39 = 8,
    Sv48 = 9,
    Sv57 = 10,
    Sv64 = 11,
}

pub struct PteInfo {
    pub mode: PteMode,
    pub level: usize,
    pub size: usize,
    pub page_size: usize,
}

impl PteInfo {
    pub fn new(satp: &Satp) -> PteInfo {
        match satp.xlen {
            XLen::X32 => PteInfo {
                mode: PteMode::try_from(satp.mode() as u8).unwrap(),
                level: 2,
                size: 4,
                page_size: 4096,
            },
            XLen::X64 => {
                let mode = PteMode::try_from(satp.mode() as u8).unwrap();
                let level = match mode {
                    PteMode::Sv39 => 3,
                    PteMode::Sv48 => 4,
                    PteMode::Bare => 0,
                    _ => unreachable!()
                };
                PteInfo {
                    mode,
                    level,
                    size: 8,
                    page_size: 4096,
                }
            }
        }
    }
}

bitfield! {
#[derive(Eq,PartialEq)]
pub struct PteAttr(u8);
impl Debug;
pub v, set_v:0, 0;
pub r, set_r:1, 1;
pub w, set_w:2, 2;
pub x, set_x:3, 3;
pub u, set_u:4, 4;
pub g, set_g:5, 5;
pub a, set_a:6,6;
pub d, set_d:7, 7;
}


impl From<u8> for PteAttr {
    fn from(v: u8) -> Self {
        PteAttr(v)
    }
}

bitfield! {
pub struct Sv32Vaddr(RegT);
impl Debug;
vpn1,_:31, 22;
vpn0,_:21, 12;
offset,_:11,0;
}

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
}

bitfield! {
pub struct Sv32Paddr(RegT);
impl Debug;
ppn1,set_ppn1:33, 22;
ppn0,set_ppn0:21, 12;
offset,set_offset:11,0;
}

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
        self.0
    }
}

bitfield! {
pub struct Sv32Pte(RegT);
impl Debug;
ppn1,_:31, 20;
ppn0,_:19, 10;
rsw,_:9,8;
attr_raw,set_attr_raw:7,0;
}

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
impl Debug;
vpn2,_:38, 30;
vpn1,_:29, 21;
vpn0,_:20, 12;
offset,_:11,0;
}

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
}

bitfield! {
pub struct Sv39Paddr(RegT);
impl Debug;
ppn2,set_ppn2:55, 30;
ppn1,set_ppn1:29, 21;
ppn0,set_ppn0:20, 12;
offset,set_offset:11,0;
}

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
        self.0
    }
}

bitfield! {
pub struct Sv39Pte(RegT);
impl Debug;
ppn2,_:53, 28;
ppn1,_:27, 19;
ppn0,_:18, 10;
rsw,_:9,8;
attr_raw,set_attr_raw:7,0;
}

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
impl Debug;
vpn3,_:47, 39;
vpn2,_:38, 30;
vpn1,_:29, 21;
vpn0,_:20, 12;
offset,_:11,0;
}

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
}

bitfield! {
pub struct Sv48Paddr(RegT);
impl Debug;
ppn3,set_ppn3:55, 39;
ppn2,set_ppn2:38, 30;
ppn1,set_ppn1:29, 21;
ppn0,set_ppn0:20, 12;
offset,set_offset:11,0;
}

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
        self.0
    }
}

bitfield! {
pub struct Sv48Pte(RegT);
impl Debug;
ppn3,_:53, 37;
ppn2,_:36, 28;
ppn1,_:27, 19;
ppn0,_:18, 10;
rsw,_:9,8;
attr_raw,set_attr_raw:7,0;
}

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
    pub fn new(mode: &PteMode, addr: RegT) -> Vaddr {
        match mode {
            PteMode::Sv32 => Vaddr::Sv32(Sv32Vaddr(addr)),
            PteMode::Sv39 => Vaddr::Sv39(Sv39Vaddr(addr)),
            PteMode::Sv48 => Vaddr::Sv48(Sv48Vaddr(addr)),
            _ => panic!(format!("unsupported PteMode {:?}", mode))
        }
    }
    // pt_export!(Vaddr, pub offset, RegT);
    pt_export!(Vaddr, pub vpn, Option<RegT>, level:usize);
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
    pub fn new(mode: &PteMode, value: RegT) -> Pte {
        match mode {
            PteMode::Sv32 => Pte::Sv32(Sv32Pte(value)),
            PteMode::Sv39 => Pte::Sv39(Sv39Pte(value)),
            PteMode::Sv48 => Pte::Sv48(Sv48Pte(value)),
            _ => panic!(format!("unsupported PteMode {:?}", mode))
        }
    }
    pub fn load(info: &PteInfo, bus: &Bus, addr: u64) -> region::Result<Pte> {
        let value = match info.size {
            4 => {
                U32Access::read(bus, addr)? as RegT
            }
            8 => {
                U64Access::read(bus, addr)? as RegT
            }
            _ => unreachable!()
        };
        Ok(Pte::new(&info.mode, value))
    }

    pub fn store(&self, bus: &Bus, addr: u64) -> region::Result<()> {
        match self {
            Pte::Sv32(_) => U32Access::write(bus, addr, self.value() as u32),
            _ => U64Access::write(bus, addr, self.value() as u64)
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