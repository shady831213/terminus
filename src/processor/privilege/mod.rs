use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::prelude::{RegT, InsnT, XLen, sext};
use crate::processor::{HasCsr, ProcessorState, ProcessorCfg};
use std::rc::Rc;
use std::cell::RefCell;

pub mod m;

use m::PrivM;
pub use m::csrs::*;

pub mod s;

use s::PrivS;
pub use s::csrs::*;

pub mod u;

use u::PrivU;

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Privilege {
    U = 0,
    S = 1,
    M = 3,
}


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
            Some(PrivilegeState::U(PrivU::new(cfg, m_state.get_csrs())))
        } else {
            None
        };
        let s = if u.is_some() && cfg.extensions.contains(&'s') {
            Some(PrivilegeState::S(PrivS::new(cfg, m_state.get_csrs())))
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

    pub fn check_extension(&self, ext: char) -> Result<(), ()> {
        if self.mcsrs().misa().get() & ((1 as RegT) << ((ext as u8 - 'a' as u8) as RegT)) != 0 {
            Ok(())
        } else {
            Err(())
        }
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

    pub fn csr_privilege_check(&self, id: InsnT) -> Result<(), ()> {
        let trip_id = id & 0xfff;
        let cur_priv: u8 = self.cur.into();
        let csr_priv: u8 = ((trip_id >> 8) & 0x3) as u8;
        if cur_priv < csr_priv {
            return Err(());
        }
        Ok(())
    }

    pub fn mcsrs(&self) -> &Rc<MCsrs> {
        if let Some(PrivilegeState::M(m_state)) = &self.m {
            m_state.get_csrs()
        } else {
            unreachable!()
        }
    }

    pub fn scsrs(&self) -> Result<&Rc<SCsrs>, ()> {
        if let Some(PrivilegeState::S(s_state)) = &self.s {
            Ok(s_state.get_csrs())
        } else {
            Err(())
        }
    }

    pub fn pending_interrupts(&self) -> RegT {
        let csrs = self.mcsrs();
        let pendings = csrs.mip().get() & csrs.mie().get();
        if pendings == 0 {
            return 0;
        }
        let mie = csrs.mstatus().mie();
        let sie = csrs.mstatus().sie();
        let deleg = csrs.mideleg().get();
        let m_enabled = self.cur != Privilege::M || (self.cur == Privilege::M && mie == 1);
        let m_pendings = pendings & !deleg & sext(m_enabled as RegT, 1);
        let s_enabled = self.cur == Privilege::U || (self.cur == Privilege::S && sie == 1);
        let s_pendings = pendings & deleg & sext(s_enabled as RegT, 1);

        //m_pendings > s_pendings
        if m_pendings == 0 {
            s_pendings
        } else {
            m_pendings
        }
    }

    fn get_priv_by_csr_idx(&self, id: InsnT) -> Result<&Option<PrivilegeState>, ()> {
        let csr_priv: u8 = ((id >> 8) & 0x3) as u8;
        match csr_priv {
            0 => Ok(&self.u),
            1 => Ok(&self.s),
            3 => Ok(&self.m),
            _ => Err(())
        }
    }
}

impl HasCsr for PrivilegeStates {
    fn csr_write(&self, state: &ProcessorState, addr: InsnT, value: RegT) -> Option<()> {
        if let Ok(Some(p)) = self.get_priv_by_csr_idx(addr) {
            match p {
                PrivilegeState::M(m) => {
                    m.get_csrs().misa_mut().set_ignore(value & ((1 as RegT) << (('c' as u8 - 'a' as u8) as RegT)) == 0 && state.pc().trailing_zeros() == 1);
                    m.get_csrs().write(addr as u64, value)
                }
                PrivilegeState::S(s) => {
                    s.get_csrs().satp_mut().set_forbidden(self.cur == Privilege::S && self.mcsrs().mstatus().tvm() != 0);
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
    fn csr_read(&self, state: &ProcessorState, addr: InsnT) -> Option<RegT> {
        match self.get_priv_by_csr_idx(addr) {
            Ok(Some(p)) => {
                match p {
                    PrivilegeState::M(m) => {
                        m.get_csrs().read(addr as u64)
                    }
                    PrivilegeState::S(s) => {
                        //stap
                        s.get_csrs().satp_mut().get_forbidden(self.cur == Privilege::S && self.mcsrs().mstatus().tvm() != 0);
                        s.get_csrs().read(addr as u64)
                    }
                    PrivilegeState::U(u) => {
                        let mcounter_dis = self.mcsrs().mcounteren().get() & ((1 as RegT) << (addr as RegT & 0x1f)) == 0;
                        let counter_dis = match self.cur {
                            Privilege::S => {
                                mcounter_dis
                            }
                            Privilege::U => {
                                if let Ok(scsrs) = self.scsrs() {
                                    mcounter_dis || (scsrs.scounteren().get() & ((1 as RegT) << (addr as RegT & 0x1f)) == 0)
                                } else {
                                    mcounter_dis
                                }
                            }
                            _ => false
                        };
                        u.get_csrs().cycle_mut().get_forbidden(counter_dis);
                        u.get_csrs().cycleh_mut().get_forbidden(counter_dis || state.config().xlen != XLen::X32);
                        u.get_csrs().instret_mut().get_forbidden(counter_dis);
                        u.get_csrs().instreth_mut().get_forbidden(counter_dis || state.config().xlen != XLen::X32);
                        u.get_csrs().read(addr as u64)
                    }
                }
            }
            _ => {
                None
            }
        }
    }
}

pub enum PrivilegeState {
    M(PrivM),
    S(PrivS),
    U(PrivU),
}
