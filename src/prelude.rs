pub use crate::global::*;
pub use terminus_vault::*;
use crate::processor::Processor;
use crate::processor::trap::Exception;
terminus_insn!(InsnT, Processor, Exception);