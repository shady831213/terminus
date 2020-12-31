use std::rc::Rc;
use crate::processor::ProcessorCfg;
use std::ops::Deref;

pub mod csrs;
use csrs::*;
use super::PrivM;

pub struct PrivS {
    csrs: Rc<SCsrs>,
}
impl PrivS {
    pub fn new(cfg: &ProcessorCfg, m:&PrivM) -> PrivS {
        let s = PrivS {
            csrs:Rc::new(SCsrs::new(cfg.xlen.len()))
        };
        s.csrs.sstatus_mut().as_s_priv();
        //deleg sstatus to mstatus
        macro_rules! deleg_sstatus_set {
                    ($setter:ident, $transform:ident) => {
                        s.csrs.sstatus_mut().$transform({
                        let csrs = (*m).clone();
                            move |field| {
                                csrs.mstatus_mut().$setter(field);
                                0
                            }
                        });
                    }
                };
        macro_rules! deleg_sstatus_get {
                    ($getter:ident, $transform:ident) => {
                        s.csrs.sstatus_mut().$transform({
                        let csrs = (*m).clone();
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
                        s.csrs.sip_mut().$transform({
                        let csrs =  (*m).clone();
                            move |_| {
                                csrs.mideleg().$getter() & csrs.mip().$getter()
                            }
                        });
                    }
                };

        s.csrs.sip_mut().set_ssip_transform({
            let csrs = (*m).clone();
            move |field| {
                if csrs.mideleg().ssip() == 1 {
                    csrs.mip_mut().set_ssip(field)
                }
                0
            }
        });

        deleg_sip_get!(usip, usip_transform);
        deleg_sip_get!(ssip, ssip_transform);
        deleg_sip_get!(utip, utip_transform);
        deleg_sip_get!(stip, stip_transform);
        deleg_sip_get!(ueip, ueip_transform);
        deleg_sip_get!(seip, seip_transform);

        //deleg sie to mie
        macro_rules! deleg_sie_get {
                    ($deleg_getter:ident, $getter:ident, $transform:ident) => {
                        s.csrs.sie_mut().$transform({
                        let csrs = (*m).clone();
                            move |_| {
                                csrs.mideleg().$deleg_getter() & csrs.mie().$getter()
                            }
                        });
                    }
                };
        macro_rules! deleg_sie_set {
                    ($deleg_getter:ident, $setter:ident, $transform:ident) => {
                        s.csrs.sie_mut().$transform({
                        let csrs = (*m).clone();
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
        s
    }
}

impl Deref for PrivS {
    type Target = Rc<SCsrs>;
    fn deref(&self) -> &Self::Target {
        &self.csrs
    }
}