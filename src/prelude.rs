pub use crate::global::*;
use crate::processor::trap::Exception;
use crate::processor::Processor;
pub use terminus_vault::*;

terminus_insn!(InsnT, Processor, Exception);
