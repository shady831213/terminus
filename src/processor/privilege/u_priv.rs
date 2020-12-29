use super::{PrivilegeCode, Privilege, PrivilegeMode};

pub struct PrivU {}

impl PrivilegeCode for PrivU{
    fn code(&self) -> Privilege {
        Privilege::U
    }
}
impl PrivilegeMode for PrivU{}