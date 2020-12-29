use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::prelude::{RegT, InsnT};
use crate::processor::{HasCsr, ProcessorState, ProcessorCfg};
use std::rc::Rc;
use std::cell::RefCell;

pub mod m;

use m::PrivM;

pub mod s;

use s::PrivS;

pub mod u;

use u::PrivU;

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Privilege {
    U = 0,
    S = 1,
    M = 3,
}


trait PrivilegeCode {
    fn code(&self) -> Privilege;
}

trait PrivilegeMode: PrivilegeCode {}

pub struct PrivilegeStates {
    m: Option<PrivilegeState>,
    s: Option<PrivilegeState>,
    u: Option<PrivilegeState>,
    cur: Privilege,
}

impl PrivilegeStates {
    pub fn new(cfg: &ProcessorCfg) -> PrivilegeStates {
        let m_state = PrivM::new(cfg);
        let u = if cfg.extensions.contains(&'u') {
            let u_state = PrivU::new(cfg);
            u_state.get_csrs().instret_mut().instret_transform({
                let csrs = m_state.get_csrs().clone();
                move |_| {
                    csrs.minstret().get()
                }
            }
            );
            u_state.get_csrs().instreth_mut().instret_transform({
                let csrs = m_state.get_csrs().clone();
                move |_| {
                    csrs.minstreth().get()
                }
            }
            );
            Some(PrivilegeState::U(u_state))
        } else {
            None
        };
        let s = if u.is_some() && cfg.extensions.contains(&'s') {
            let s_state = PrivS::new(cfg);
            //deleg sstatus to mstatus
            macro_rules! deleg_sstatus_set {
                    ($setter:ident, $transform:ident) => {
                        s_state.get_csrs().sstatus_mut().$transform({
                        let csrs = m_state.get_csrs().clone();
                            move |field| {
                                csrs.mstatus_mut().$setter(field);
                                0
                            }
                        });
                    }
                };
            macro_rules! deleg_sstatus_get {
                    ($getter:ident, $transform:ident) => {
                        s_state.get_csrs().sstatus_mut().$transform({
                        let csrs = m_state.get_csrs().clone();
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
                        s_state.get_csrs().sip_mut().$transform({
                        let csrs =  m_state.get_csrs().clone();
                            move |_| {
                                csrs.mideleg().$getter() & csrs.mip().$getter()
                            }
                        });
                    }
                };

            s_state.get_csrs().sip_mut().set_ssip_transform({
                let csrs = m_state.get_csrs().clone();
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
                        s_state.get_csrs().sie_mut().$transform({
                        let csrs = m_state.get_csrs().clone();
                            move |_| {
                                csrs.mideleg().$deleg_getter() & csrs.mie().$getter()
                            }
                        });
                    }
                };
            macro_rules! deleg_sie_set {
                    ($deleg_getter:ident, $setter:ident, $transform:ident) => {
                        s_state.get_csrs().sie_mut().$transform({
                        let csrs = m_state.get_csrs().clone();
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

            Some(PrivilegeState::S(s_state))
        } else {
            m_state.get_csrs().mstatus_mut().set_tvm_transform(|_| { 0 });
            m_state.get_csrs().mstatus_mut().set_tsr_transform(|_| { 0 });
            None
        };

        //privilege_level config
        if u.is_none() {
            let m: u8 = Privilege::M.into();
            m_state.get_csrs().mstatus_mut().set_mpp(m as RegT);
            m_state.get_csrs().mstatus_mut().set_mpp_transform(move |_| {
                m as RegT
            });
            m_state.get_csrs().mstatus_mut().set_tw_transform(|_| { 0 });
        } else if s.is_none() {
            m_state.get_csrs().mstatus_mut().set_mpp_transform(|mpp| {
                if mpp != 0 {
                    let m: u8 = Privilege::M.into();
                    m as RegT
                } else {
                    0
                }
            });
        }

        let m = Some(PrivilegeState::M(m_state));
        PrivilegeStates {
            m,
            s,
            u,
            cur: Privilege::M,
        }
    }

    pub fn get_priv(&self, p: Privilege) -> &Option<PrivilegeState> {
        match p {
            Privilege::M => &self.m,
            Privilege::S => &self.s,
            Privilege::U => &self.u,
        }
    }

    pub fn get_priv_mut(&mut self, p: Privilege) -> &mut Option<PrivilegeState> {
        match p {
            Privilege::M => &mut self.m,
            Privilege::S => &mut self.s,
            Privilege::U => &mut self.u,
        }
    }

    pub fn set_priv(&mut self, p: Privilege) {
        match p {
            Privilege::S if self.s.is_none() => { self.cur = Privilege::M }
            Privilege::U if self.u.is_none() => { self.cur = Privilege::M }
            _ => self.cur = p,
        }
    }

    pub fn cur_privilege(&self) -> &Privilege {
        &self.cur
    }

    pub fn delegate_insns_cnt(&self, cnt: &Rc<RefCell<u64>>) {
        if let Some(PrivilegeState::M(m_state)) = &self.m {
            m_state.get_csrs().minstret_mut().instret_transform({
                let count = cnt.clone();
                move |_| {
                    *count.borrow() as RegT
                }
            }
            );
            m_state.get_csrs().minstreth_mut().instret_transform({
                let count = cnt.clone();
                move |_| {
                    (*count.borrow() >> 32) as RegT
                }
            }
            );
        }
    }

    pub fn csr_privilege_check(&self, id: InsnT) -> Result<(), InsnT> {
        let trip_id = id & 0xfff;
        let cur_priv: u8 = self.cur.into();
        let csr_priv: u8 = ((trip_id >> 8) & 0x3) as u8;
        if cur_priv < csr_priv {
            return Err(id);
        }
        Ok(())
    }

    fn get_priv_by_csr_idx(&self, id: InsnT) -> Result<&Option<PrivilegeState>, InsnT> {
        let csr_priv: u8 = ((id >> 8) & 0x3) as u8;
        match csr_priv {
            0 => Ok(&self.u),
            1 => Ok(&self.s),
            3 => Ok(&self.m),
            _ => Err(id)
        }
    }
}

impl HasCsr for PrivilegeStates {
    fn csr_write(&self, state: &ProcessorState, addr: InsnT, value: RegT) -> Option<()> {
        let m_csrs = if let Some(PrivilegeState::M(m_state)) = &self.m {m_state.get_csrs()} else {unreachable!()};
        if let Ok(Some(p)) = self.get_priv_by_csr_idx(addr) {
            match p {
                PrivilegeState::M(m) => {
                    if value & ((1 as RegT) << (('c' as u8 - 'a' as u8) as RegT)) == 0 && addr == 0x301 && state.pc().trailing_zeros() == 1 {
                        return Some(());
                    }
                    m.get_csrs().write(addr as u64, value)
                }
                PrivilegeState::S(s) => {
                    //stap
                    if addr == 0x180 && self.cur == Privilege::S && m_csrs.mstatus().tvm() != 0 {
                        return None;
                    }
                    s.get_csrs().write(addr as u64, value)
                }
                PrivilegeState::U(u) => {
                    u.get_csrs().write(addr as u64, value)
                }
            }
        } else {
            None
        }
    }
    fn csr_read(&self, _: &ProcessorState, addr: InsnT) -> Option<RegT> {
        let m_csrs = if let Some(PrivilegeState::M(m_state)) = &self.m {m_state.get_csrs()} else {unreachable!()};
        let addr_high = addr & 0xf0;
        if let Ok(Some(p)) = self.get_priv_by_csr_idx(addr) {
            match p {
                PrivilegeState::M(m) => {
                    m.get_csrs().read(addr as u64)
                }
                PrivilegeState::S(s) => {
                    //stap
                    if addr == 0x180 && self.cur == Privilege::S && m_csrs.mstatus().tvm() != 0 {
                        return None;
                    }
                    s.get_csrs().read(addr as u64)
                }
                PrivilegeState::U(u) => {
                    if addr_high == 0x80 || addr_high == 0x90 || addr_high == 0x00 || addr_high == 0x10 {
                        match self.cur {
                            Privilege::S if m_csrs.mcounteren().get() & ((1 as RegT) << (addr as RegT & 0x1f)) == 0 => {
                                return None;
                            }
                            Privilege::U => {
                                if m_csrs.mcounteren().get() & ((1 as RegT) << (addr as RegT & 0x1f)) == 0 {
                                    return None;
                                }
                                if let Some(PrivilegeState::S(s_state)) = &self.s {
                                    if s_state.get_csrs().scounteren().get() & ((1 as RegT) << (addr as RegT & 0x1f)) == 0 {
                                        return None;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    u.get_csrs().read(addr as u64)
                }
            }
        } else {
            None
        }
    }
}

pub enum PrivilegeState {
    M(PrivM),
    S(PrivS),
    U(PrivU),
}
