use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::prelude::{RegT, InsnT, XLen, sext};
use crate::processor::{HasCsr, ProcessorState, ProcessorCfg};
use std::rc::Rc;
use std::cell::RefCell;
use std::convert::TryFrom;

mod m;

pub use m::PrivM;
pub use m::csrs::*;

mod s;

pub use s::PrivS;
pub use s::csrs::*;

mod u;

pub use u::PrivU;
pub use u::csrs::*;

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Privilege {
    U = 0,
    S = 1,
    M = 3,
}


pub struct PrivilegeStates {
    m: PrivM,
    s: Option<PrivS>,
    u: Option<PrivU>,
    cur: Privilege,
}

impl PrivilegeStates {
    pub fn new(cfg: &ProcessorCfg) -> PrivilegeStates {
        let m_state = PrivM::new(cfg);
        let u = if cfg.extensions.contains(&'u') {
            Some(PrivU::new(cfg, &m_state))
        } else {
            None
        };
        let s = if u.is_some() && cfg.extensions.contains(&'s') {
            Some(PrivS::new(cfg, &m_state))
        } else {
            m_state.mstatus_mut().set_tvm_transform(|_| { 0 });
            m_state.mstatus_mut().set_tsr_transform(|_| { 0 });
            None
        };

        //privilege_level config
        if u.is_none() {
            let m: u8 = Privilege::M.into();
            m_state.mstatus_mut().set_mpp(m as RegT);
            m_state.mstatus_mut().set_mpp_transform(move |_| {
                m as RegT
            });
            m_state.mstatus_mut().set_tw_transform(|_| { 0 });
        } else if s.is_none() {
            m_state.mstatus_mut().set_mpp_transform(|mpp| {
                if mpp != 0 {
                    let m: u8 = Privilege::M.into();
                    m as RegT
                } else {
                    0
                }
            });
        }

        PrivilegeStates {
            m: m_state,
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
        if self.m().misa().get() & ((1 as RegT) << ((ext as u8 - 'a' as u8) as RegT)) != 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn delegate_insns_cnt(&self, cnt: &Rc<RefCell<u64>>) {
        self.m.minstret_mut().instret_transform({
            let count = cnt.clone();
            move |_| {
                *count.borrow() as RegT
            }
        }
        );
        self.m.minstreth_mut().instret_transform({
            let count = cnt.clone();
            move |_| {
                (*count.borrow() >> 32) as RegT
            }
        }
        );
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

    pub fn m(&self) -> &PrivM {
        &self.m
    }

    pub fn s(&self) -> Option<&PrivS> {
        self.s.as_ref()
    }

    pub fn u(&self) -> Option<&PrivU> {
        self.u.as_ref()
    }

    pub fn pending_interrupts(&self) -> RegT {
        let m = self.m();
        let pendings = m.mip().get() & m.mie().get();
        if pendings == 0 {
            return 0;
        }
        let mie = m.mstatus().mie();
        let sie = m.mstatus().sie();
        let deleg = m.mideleg().get();
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

    fn delegate_priv(&self, code: RegT, int_flag: bool) -> Privilege {
        let deleg = if int_flag {
            self.m().mideleg().get()
        } else {
            self.m().medeleg().get()
        };
        //deleg to s-mode
        let degeged = self.cur != Privilege::M && (deleg >> code) & 1 == 1;
        if degeged {
            Privilege::S
        } else {
            Privilege::M
        }
    }

    pub fn trap_enter(&self, state: &ProcessorState, code: RegT, int_flag: bool, val: RegT) -> (RegT, Privilege) {
        let tgt_privilege = self.delegate_priv(code, int_flag);
        let (tvec, mut cause, mut epc, mut tval) = match &tgt_privilege {
            Privilege::M => {
                (self.m().mtvec(),
                 self.m().mcause_mut(),
                 self.m().mepc_mut(),
                 self.m().mtval_mut()
                )
            }
            Privilege::S => {
                (self.s().unwrap().stvec(),
                 self.s().unwrap().scause_mut(),
                 self.s().unwrap().sepc_mut(),
                 self.s().unwrap().stval_mut()
                )
            }
            _ => unreachable!()
        };
        let pc = tvec.get_trap_pc(code, int_flag);
        cause.set_cause(code, int_flag);
        if int_flag {
            epc.set(*state.next_pc());
        } else {
            epc.set(*state.pc());
        }
        tval.set(val);

        self.m().mstatus_mut().push_privilege(&tgt_privilege, &self.cur);
        (pc, tgt_privilege)
    }

    pub fn trap_return(&self, cur_privilege: &Privilege) -> (RegT, Privilege) {
        let epc = match cur_privilege {
            Privilege::M => self.m().mepc(),
            Privilege::S => self.s().unwrap().sepc(),
            _ => unreachable!()
        };
        let xpp = self.m().mstatus_mut().pop_privilege(cur_privilege);
        let pc = if self.check_extension('c').is_err() {
            (epc.get() >> 2) << 2
        } else {
            epc.get()
        };
        (pc, xpp)
    }

    fn get_priv_by_csr_idx(&self, id: InsnT) -> Result<Privilege, ()> {
        let csr_priv: u8 = ((id >> 8) & 0x3) as u8;
        Privilege::try_from(csr_priv).map_err(|_| { () })
    }
}

impl HasCsr for PrivilegeStates {
    fn csr_write(&self, state: &ProcessorState, addr: InsnT, value: RegT) -> Option<()> {
        if let Ok(p) = self.get_priv_by_csr_idx(addr) {
            match p {
                Privilege::M => {
                    self.m().misa_mut().set_ignore(value & ((1 as RegT) << (('c' as u8 - 'a' as u8) as RegT)) == 0 && state.pc().trailing_zeros() == 1);
                    self.m().write(addr as u64, value)
                }
                Privilege::S => {
                    self.s().map(|s| {
                        s.satp_mut().set_forbidden(self.cur == Privilege::S && self.m().mstatus().tvm() != 0);
                        s.write(addr as u64, value)
                    }).flatten()
                }
                Privilege::U => {
                    self.u().map(|u| { u.write(addr as u64, value) }).flatten()
                }
            }
        } else {
            None
        }
    }
    fn csr_read(&self, state: &ProcessorState, addr: InsnT) -> Option<RegT> {
        if let Ok(p) = self.get_priv_by_csr_idx(addr) {
            match p {
                Privilege::M => {
                    self.m().read(addr as u64)
                }
                Privilege::S => {
                    self.s().map(|s| {
                        s.satp_mut().get_forbidden(self.cur == Privilege::S && self.m().mstatus().tvm() != 0);
                        s.read(addr as u64)
                    }).flatten()
                }
                Privilege::U => {
                    self.u().map(|u| {
                        let mcounter_dis = self.m().mcounteren().get() & ((1 as RegT) << (addr as RegT & 0x1f)) == 0;
                        let counter_dis = match self.cur {
                            Privilege::S => {
                                mcounter_dis
                            }
                            Privilege::U => {
                                if let Some(s) = self.s() {
                                    mcounter_dis || (s.scounteren().get() & ((1 as RegT) << (addr as RegT & 0x1f)) == 0)
                                } else {
                                    mcounter_dis
                                }
                            }
                            _ => false
                        };
                        u.cycle_mut().get_forbidden(counter_dis);
                        u.cycleh_mut().get_forbidden(counter_dis || state.config().xlen != XLen::X32);
                        u.instret_mut().get_forbidden(counter_dis);
                        u.instreth_mut().get_forbidden(counter_dis || state.config().xlen != XLen::X32);
                        u.read(addr as u64)
                    }).flatten()
                }
            }
        } else {
            None
        }
    }
}
