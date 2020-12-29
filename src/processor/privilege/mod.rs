use num_enum::{IntoPrimitive, TryFromPrimitive};
use super::ProcessorCfg;

mod m_priv;
use m_priv::PrivM;
mod s_priv;
use s_priv::PrivS;
mod u_priv;
use u_priv::PrivU;

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

trait PrivilegeMode:PrivilegeCode{}

pub struct PrivilegeStates {
    m: Option<PrivilegeState>,
    s: Option<PrivilegeState>,
    u: Option<PrivilegeState>,
    cur:Privilege,
}

impl PrivilegeStates {
    pub fn new(cfg:&ProcessorCfg)-> PrivilegeStates {
        let m = Some(PrivilegeState::M(PrivM{}));
        let u = if cfg.extensions.contains(&'u') {
            Some(PrivilegeState::U(PrivU{}))
        } else {
            None
        };
        let s = if u.is_some() && cfg.extensions.contains(&'s') {
            Some(PrivilegeState::S(PrivS{}))
        } else {
            None
        };
        PrivilegeStates {
            m,
            s,
            u,
            cur:Privilege::M
        }
    }

    pub fn get_priv(&self, p:Privilege) -> &Option<PrivilegeState> {
        match p {
            Privilege::M => &self.m,
            Privilege::S => &self.s,
            Privilege::U => &self.u,
        }
    }

    pub fn get_priv_mut(&mut self, p:Privilege) -> &mut Option<PrivilegeState> {
        match p {
            Privilege::M => &mut self.m,
            Privilege::S => &mut self.s,
            Privilege::U => &mut self.u,
        }
    }

    pub fn set_priv(&mut self, p:Privilege) {
        match p {
            Privilege::S if self.s.is_none() =>  {self.cur = Privilege::M}
            Privilege::U if self.u.is_none() =>  {self.cur = Privilege::M},
            _ => self.cur = p,
        }
    }

    pub fn cur_privilege(&self) -> &Privilege {
        &self.cur
    }
}

pub enum PrivilegeState{
    M(PrivM),
    S(PrivS),
    U(PrivU),
}
