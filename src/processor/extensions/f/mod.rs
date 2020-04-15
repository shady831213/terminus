use crate::processor::ProcessorState;
use std::cell::{RefCell, Ref};
use std::rc::Rc;
use crate::processor::extensions::HasCsr;
use std::any::Any;
use terminus_global::RegT;
use crate::processor::extensions::i::csrs::*;

mod insns;
pub mod csrs;

use csrs::FCsrs;
use std::ops::Deref;

pub type FRegT = u128;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FLen {
    F32,
    F64,
    F128,
}

impl FLen {
    pub fn len(&self) -> usize {
        match self {
            FLen::F32 => 32,
            FLen::F64 => 64,
            FLen::F128 => 128,
        }
    }

    pub fn size(&self) -> usize {
        self.len() >> 3
    }

    pub fn mask(&self) -> FRegT {
        match self {
            FLen::F32 => ((1 as FRegT) << (self.len() as FRegT)) - 1,
            FLen::F64 => ((1 as FRegT) << (self.len() as FRegT)) - 1,
            FLen::F128 => -1i128 as FRegT
        }
    }
}

pub struct ExtensionF {
    pub flen: FLen,
    freg: RefCell<[FRegT; 32]>,
    csrs: Rc<FCsrs>,
    dirty: Rc<RefCell<RegT>>,
}

impl ExtensionF {
    pub fn new(state: &ProcessorState) -> ExtensionF {
        let mut e = ExtensionF {
            flen: FLen::F32,
            freg: RefCell::new([0 as FRegT; 32]),
            csrs: Rc::new(FCsrs::new(state.config().xlen)),
            dirty: Rc::new(RefCell::new(0)),
        };

        if state.config().extensions.contains(&'q') {
            e.flen = FLen::F128
        } else if state.config().extensions.contains(&'d') {
            e.flen = FLen::F64
        }

        //map dirty to mstatus.fs
        state.csrs::<ICsrs>().unwrap().mstatus_mut().set_fs_transform(
            {
                let dirty = e.dirty.clone();
                move |value| {
                    *dirty.borrow_mut() = value & 0x3;
                    0
                }
            }
        );
        state.csrs::<ICsrs>().unwrap().mstatus_mut().fs_transform(
            {
                let dirty = e.dirty.clone();
                move |_| {
                    *dirty.deref().borrow()
                }
            }
        );
        //deleg frm and fflags to fcsr
        macro_rules! deleg_fcsr_set {
                    ($src:ident, $setter:ident, $transform:ident) => {
                        e.csrs.$src().$transform({
                        let csrs = e.csrs.clone();
                            move |field| {
                                csrs.fcsr_mut().$setter(field);
                                0
                            }
                        });
                    }
                };
        macro_rules! deleg_fcsr_get {
                    ($src:ident, $getter:ident, $transform:ident) => {
                        e.csrs.$src().$transform({
                        let csrs = e.csrs.clone();
                            move |_| {
                                csrs.fcsr().$getter()
                            }
                        });
                    }
                };
        macro_rules! deleg_fcsr {
                    ($src:ident, $getter:ident, $get_transform:ident, $setter:ident, $set_transform:ident) => {
                        deleg_fcsr_get!($src, $getter, $get_transform);
                        deleg_fcsr_set!($src, $setter, $set_transform);
                    }
                };
        deleg_fcsr!(frm_mut, frm, frm_transform, set_frm, set_frm_transform);
        deleg_fcsr!(fflags_mut, nx, nx_transform, set_nx, set_nx_transform);
        deleg_fcsr!(fflags_mut, uf, uf_transform, set_uf, set_uf_transform);
        deleg_fcsr!(fflags_mut, of, of_transform, set_of, set_of_transform);
        deleg_fcsr!(fflags_mut, dz, dz_transform, set_dz, set_dz_transform);
        deleg_fcsr!(fflags_mut, nv, nv_transform, set_nv, set_nv_transform);

        e
    }

    pub fn freg(&self, id: RegT) -> FRegT {
        let trip_id = id & 0x1f;
        (*self.freg.borrow())[trip_id as usize]
    }

    pub fn set_freg(&self, id: RegT, value: FRegT) {
        let trip_id = id & 0x1f;
        *self.dirty.borrow_mut() = 0x3;
        (*self.freg.borrow_mut())[trip_id as usize] = value
    }

    pub fn dirty(&self) -> RegT {
        *self.dirty.deref().borrow()
    }

    pub fn fregs(&self) -> Ref<'_, [FRegT; 32]> {
        self.freg.borrow()
    }
}

impl HasCsr for ExtensionF {
    fn csrs(&self) -> Option<Rc<dyn Any>> {
        Some(self.csrs.clone() as Rc<dyn Any>)
    }
    fn csr_write(&self, _: &ProcessorState, addr: RegT, value: RegT) -> Option<()> {
        *self.dirty.borrow_mut() = 0x3;
        self.csrs.write(addr, value)
    }
    fn csr_read(&self, _: &ProcessorState, addr: RegT) -> Option<RegT> {
        if self.dirty() == 0 {
            None
        } else {
            self.csrs.read(addr)
        }
    }
}
