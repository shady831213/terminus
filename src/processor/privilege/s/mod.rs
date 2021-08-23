use crate::processor::ProcessorCfg;
use paste::paste;
use std::ops::Deref;
use std::rc::Rc;
pub mod csrs;
use super::PrivM;
use csrs::*;

pub struct PrivS {
    csrs: Rc<SCsrs>,
}
impl PrivS {
    pub fn new(cfg: &ProcessorCfg, m: &PrivM) -> PrivS {
        let s = PrivS {
            csrs: Rc::new(SCsrs::new(cfg.xlen.len())),
        };
        s.csrs.sstatus_mut().as_s_priv();
        //deleg sstatus to mstatus
        macro_rules! deleg_sstatus {
            ($field:ident) => {
                paste! {
                    s.csrs.sstatus_mut().[<set_ $field _transform>]({
                    let csrs = (*m).clone();
                        move |field| {
                            csrs.mstatus_mut().[<set_ $field>](field);
                            0
                        }
                    });
                    s.csrs.sstatus_mut().[<$field _transform>]({
                    let csrs = (*m).clone();
                        move |_| {
                            csrs.mstatus().$field()
                        }
                    });
                }
            };
        }
        deleg_sstatus!(upie);
        deleg_sstatus!(sie);
        deleg_sstatus!(upie);
        deleg_sstatus!(spie);
        deleg_sstatus!(spp);
        deleg_sstatus!(fs);
        deleg_sstatus!(xs);
        deleg_sstatus!(sum);
        deleg_sstatus!(mxr);
        deleg_sstatus!(sd);
        deleg_sstatus!(uxl);

        //deleg sip to mip
        macro_rules! deleg_sip {
            ($field:ident) => {
                paste! {
                    s.csrs.sip_mut().[<$field _transform>]({
                    let csrs =  (*m).clone();
                        move |_| {
                            csrs.mideleg().$field() & csrs.mip().$field()
                        }
                    });
                }
            };
        }
        s.csrs.sip_mut().set_ssip_transform({
            let csrs = (*m).clone();
            move |field| {
                if csrs.mideleg().ssip() == 1 {
                    csrs.mip_mut().set_ssip(field)
                }
                0
            }
        });

        deleg_sip!(usip);
        deleg_sip!(ssip);
        deleg_sip!(utip);
        deleg_sip!(stip);
        deleg_sip!(ueip);
        deleg_sip!(seip);

        //deleg sie to mie
        macro_rules! deleg_sie {
            ($deleg_filed:ident, $field:ident) => {
                paste! {
                    s.csrs.sie_mut().[<$field _transform>]({
                    let csrs = (*m).clone();
                        move |_| {
                            csrs.mideleg().$deleg_filed() & csrs.mie().$field()
                        }
                    });
                    s.csrs.sie_mut().[<set_ $field _transform>]({
                    let csrs = (*m).clone();
                        move |field| {
                            if csrs.mideleg().$deleg_filed() == 1 {
                                csrs.mie_mut().[<set_ $field>](field)
                            }
                            0
                        }
                    });
                }
            };
        }
        deleg_sie!(usip, usie);
        deleg_sie!(ssip, ssie);
        deleg_sie!(utip, utie);
        deleg_sie!(stip, stie);
        deleg_sie!(ueip, ueie);
        deleg_sie!(seip, seie);
        s
    }
}

impl Deref for PrivS {
    type Target = Rc<SCsrs>;
    fn deref(&self) -> &Self::Target {
        &self.csrs
    }
}
