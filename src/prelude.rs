pub mod insn {
    pub use terminus_global::*;
    pub use terminus_vault::*;
    pub use crate::processor::{Processor, Privilege, PrivilegeLevel};
    pub use crate::processor::trap::Exception;
    terminus_insn!(InsnT, Processor, Exception);
}
pub use insn::*;