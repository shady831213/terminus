use terminus_global::*;
use std::rc::Rc;
use std::any::Any;
use crate::processor::extensions::HasCsr;

mod insns;
pub mod csrs;

use csrs::ICsrs;
use crate::processor::{ProcessorCfg, PrivilegeLevel, Privilege};

pub struct ExtensionI {
    csrs: Rc<ICsrs>,
}

impl ExtensionI {
    pub fn new(cfg: &ProcessorCfg) -> ExtensionI {
        let e = ExtensionI {
            csrs: Rc::new(ICsrs::new(cfg.xlen))
        };
        //no debug
        e.csrs.tselect_mut().set(0xffff_ffff_ffff_ffff);
        //mstatus
        //sd bit
        e.csrs.mstatus_mut().sd_transform({
            let csrs = e.csrs.clone();
            move |_| {
                if csrs.mstatus().fs() == 0x3 && csrs.mstatus().xs() == 0x3 {
                    1
                } else {
                    0
                }
            }
        }
        );
        //deleg sstatus to mstatus
        macro_rules! deleg_sstatus_set {
                    ($setter:ident, $transform:ident) => {
                        e.csrs.sstatus_mut().$transform({
                        let csrs = e.csrs.clone();
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
                        let csrs = e.csrs.clone();
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
                        let csrs = e.csrs.clone();
                            move |_| {
                                csrs.mideleg().$getter() & csrs.mip().$getter()
                            }
                        });
                    }
                };
        e.csrs.sip_mut().set_ssip_transform({
            let csrs = e.csrs.clone();
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
                        let csrs = e.csrs.clone();
                            move |_| {
                                csrs.mideleg().$deleg_getter() & csrs.mie().$getter()
                            }
                        });
                    }
                };
        macro_rules! deleg_sie_set {
                    ($deleg_getter:ident, $setter:ident, $transform:ident) => {
                        e.csrs.sie_mut().$transform({
                        let csrs = e.csrs.clone();
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
        //xlen config
        match cfg.xlen {
            XLen::X32 => {
                e.csrs.misa_mut().set_mxl(1);
            }
            XLen::X64 => {
                e.csrs.misa_mut().set_mxl(2);
                e.csrs.mstatus_mut().set_uxl(2);
                e.csrs.mstatus_mut().set_sxl(2);
                e.csrs.sstatus_mut().set_uxl(2);
            }
        }

        //privilege_level config
        match cfg.privilege_level() {
            PrivilegeLevel::MSU => {}
            PrivilegeLevel::MU => {
                e.csrs.mstatus_mut().set_mpp_transform(|mpp| {
                    if mpp != 0 {
                        let m: u8 = Privilege::M.into();
                        m as RegT
                    } else {
                        0
                    }
                });
                e.csrs.mstatus_mut().set_spp_transform(|_| { 0 });
                e.csrs.mstatus_mut().set_tvm_transform(|_| { 0 });
                e.csrs.mstatus_mut().set_tsr_transform(|_| { 0 });
            }
            PrivilegeLevel::M => {
                let m: u8 = Privilege::M.into();
                e.csrs.mstatus_mut().set_mpp(m as RegT);
                e.csrs.mstatus_mut().set_mpp_transform(move |_| {
                    m as RegT
                });
                e.csrs.mstatus_mut().set_spp_transform(|_| { 0 });
                e.csrs.mstatus_mut().set_tvm_transform(|_| { 0 });
                e.csrs.mstatus_mut().set_tsr_transform(|_| { 0 });
                e.csrs.mstatus_mut().set_tw_transform(|_| { 0 });
            }
        }

        e
    }
}

impl HasCsr for ExtensionI {
    fn csrs(&self) -> Option<Rc<dyn Any>> {
        Some(self.csrs.clone() as Rc<dyn Any>)
    }
    fn csr_write(&self, addr: RegT, value: RegT) -> Option<()> {
        self.csrs.write(addr, value)
    }
    fn csr_read(&self, addr: RegT) -> Option<RegT> {
        self.csrs.read(addr)
    }
}

