pub mod insn {
    pub use terminus_global::*;
    pub use terminus_macros::*;
    pub use terminus_proc_macros::*;
    pub use crate::processor::{Processor, Privilege, PrivilegeLevel};
    pub use crate::processor::trap::Exception;
    terminus_insn!(InsnT, Processor, Exception);
}
pub use insn::*;