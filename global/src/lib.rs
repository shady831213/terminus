pub type InsnT = u32;
pub fn insn_len()->usize {std::mem::size_of::<InsnT>()}