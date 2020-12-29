use super::{PrivilegeCode, Privilege, PrivilegeMode};
pub struct PrivM {}

impl PrivilegeCode for PrivM{
    fn code(&self) -> Privilege {
        Privilege::M
    }
}
impl PrivilegeMode for PrivM{}