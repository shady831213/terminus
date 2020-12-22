pub use linkme::*;
pub use terminus_global::*;
pub use terminus_macros::*;
pub use terminus_proc_macros::*;
pub use crate::processor::{Processor, Privilege, PrivilegeLevel};
pub use crate::processor::trap::Exception;
pub mod insn {
    use terminus_macros::*;
    use crate::processor::Processor;
    use crate::processor::trap::Exception;
    use terminus_global::*;
    terminus_insn!(InsnT, Processor, Exception);
}
pub use insn::*;