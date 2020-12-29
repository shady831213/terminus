use super::{PrivilegeCode, Privilege, PrivilegeMode};

pub struct PrivS {}

impl PrivilegeCode for PrivS{
    fn code(&self) -> Privilege {
        Privilege::S
    }
}
impl PrivilegeMode for PrivS{}