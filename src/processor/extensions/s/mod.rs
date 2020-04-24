use std::rc::Rc;
use crate::processor::{ProcessorState, Privilege};
use crate::processor::extensions::{HasCsr, NoStepCb};
use terminus_global::RegT;
use std::cell::RefCell;

mod insns;
pub mod csrs;

use csrs::*;
use std::ops::Deref;

pub struct ExtensionS {
    csrs: Rc<SCsrs>,
    tvm: Rc<RefCell<bool>>,
    tsr: Rc<RefCell<bool>>,

}

impl ExtensionS {
    pub fn new(state: &ProcessorState) -> ExtensionS {
        let e = ExtensionS {
            csrs: Rc::new(SCsrs::new(state.config().xlen)),
            tvm: Rc::new(RefCell::new(false)),
            tsr: Rc::new(RefCell::new(false)),
        };
        let icsrs = state.icsrs();
        //map tvm and tsr
        icsrs.mstatus_mut().set_tvm_transform({
            let tvm = e.tvm.clone();
            move |value| {
                *tvm.borrow_mut() = value & 0x1 == 1;
                0
            }
        });
        icsrs.mstatus_mut().tvm_transform({
            let tvm = e.tvm.clone();
            move |_| {
                *tvm.deref().borrow() as RegT
            }
        });
        icsrs.mstatus_mut().set_tsr_transform({
            let tsr = e.tsr.clone();
            move |value| {
                *tsr.borrow_mut() = value & 0x1 == 1;
                0
            }
        });
        icsrs.mstatus_mut().tsr_transform({
            let tsr = e.tsr.clone();
            move |_| {
                *tsr.deref().borrow() as RegT
            }
        });
        //deleg sstatus to mstatus
        macro_rules! deleg_sstatus_set {
                    ($setter:ident, $transform:ident) => {
                        e.csrs.sstatus_mut().$transform({
                        let csrs = icsrs.clone();
                            move |field| {
                                csrs.mstatus_mut().$setter(field);
                                0
                            }
                        });
                    }
                };
        macro_rules! deleg_sstatus_get {
                    ($getter:ident, $transform:ident) => {
                        e.csrs.sstatus_mut().$transform({
                        let csrs = icsrs.clone();
                            move |_| {
                                csrs.mstatus().$getter()
                            }
                        });
                    }
                };
        macro_rules! deleg_sstatus {
                    ($getter:ident, $get_transform:ident, $setter:ident, $set_transform:ident) => {
                        deleg_sstatus_get!($getter, $get_transform);
                        deleg_sstatus_set!($setter, $set_transform);
                    }
                };
        deleg_sstatus!(upie, upie_transform, set_upie, set_upie_transform);
        deleg_sstatus!(sie, sie_transform, set_sie, set_sie_transform);
        deleg_sstatus!(upie, upie_transform, set_upie, set_upie_transform);
        deleg_sstatus!(spie, spie_transform, set_spie, set_spie_transform);
        deleg_sstatus!(spp, spp_transform, set_spp, set_spp_transform);
        deleg_sstatus!(fs, fs_transform, set_fs, set_fs_transform);
        deleg_sstatus!(xs, xs_transform, set_xs, set_xs_transform);
        deleg_sstatus!(sum, sum_transform, set_sum, set_sum_transform);
        deleg_sstatus!(mxr, mxr_transform, set_mxr, set_mxr_transform);
        deleg_sstatus!(sd, sd_transform, set_sd, set_sd_transform);
        deleg_sstatus!(uxl, uxl_transform, set_uxl, set_uxl_transform);

        //deleg sip to mip
        macro_rules! deleg_sip_get {
                    ($getter:ident, $transform:ident) => {
                        e.csrs.sip_mut().$transform({
                        let csrs = icsrs.clone();
                            move |_| {
                                csrs.mideleg().$getter() & csrs.mip().$getter()
                            }
                        });
                    }
                };
        e.csrs.sip_mut().set_ssip_transform({
            let csrs = icsrs.clone();
            move |field| {
                if csrs.mideleg().ssip() == 1 {
                    csrs.mip_mut().set_ssip(field)
                }
                0
            }
        }
        );
        deleg_sip_get!(usip, usip_transform);
        deleg_sip_get!(ssip, ssip_transform);
        deleg_sip_get!(utip, utip_transform);
        deleg_sip_get!(stip, stip_transform);
        deleg_sip_get!(ueip, ueip_transform);
        deleg_sip_get!(seip, seip_transform);

        //deleg sie to mie
        macro_rules! deleg_sie_get {
                    ($deleg_getter:ident, $getter:ident, $transform:ident) => {
                        e.csrs.sie_mut().$transform({
                        let csrs = icsrs.clone();
                            move |_| {
                                csrs.mideleg().$deleg_getter() & csrs.mie().$getter()
                            }
                        });
                    }
                };
        macro_rules! deleg_sie_set {
                    ($deleg_getter:ident, $setter:ident, $transform:ident) => {
                        e.csrs.sie_mut().$transform({
                        let csrs = icsrs.clone();
                            move |field| {
                                if csrs.mideleg().$deleg_getter() == 1 {
                                    csrs.mie_mut().$setter(field)
                                }
                                0
                            }
                        });
                    }
                };
        macro_rules! deleg_sie {
                    ($deleg_getter:ident, $getter:ident, $get_transform:ident, $setter:ident, $set_transform:ident) => {
                        deleg_sie_get!($deleg_getter, $getter, $get_transform);
                        deleg_sie_set!($deleg_getter, $setter, $set_transform);
                    }
                };
        deleg_sie!(usip, usie, usie_transform, set_usie, set_usie_transform);
        deleg_sie!(ssip, ssie, ssie_transform, set_ssie, set_ssie_transform);
        deleg_sie!(utip, utie, utie_transform, set_utie, set_utie_transform);
        deleg_sie!(stip, stie, stie_transform, set_stie, set_stie_transform);
        deleg_sie!(ueip, ueie, ueie_transform, set_ueie, set_ueie_transform);
        deleg_sie!(seip, seie, seie_transform, set_seie, set_seie_transform);
        e
    }

    pub fn get_csrs(&self) -> &Rc<SCsrs> {
        &self.csrs
    }
}

impl HasCsr for ExtensionS {
    fn csr_write(&self, state:&ProcessorState, addr: RegT, value: RegT) -> Option<()> {
        //stap
        if addr == 0x180 && state.privilege() == Privilege::S && *self.tvm.borrow() {
            return None;
        }
        self.csrs.write(addr, value)
    }
    fn csr_read(&self, state:&ProcessorState, addr: RegT) -> Option<RegT> {
        //stap
        if addr == 0x180 && state.privilege() == Privilege::S && *self.tvm.borrow() {
            return None;
        }
        self.csrs.read(addr)
    }
}

impl NoStepCb for ExtensionS{}

