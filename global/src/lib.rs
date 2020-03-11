pub type InsnT = u32;
pub fn insn_len()->usize {std::mem::size_of::<InsnT>()}

pub type RegT = u64;
pub fn reg_len()->usize {std::mem::size_of::<RegT>()}
