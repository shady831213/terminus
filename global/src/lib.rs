pub type InsnT = u32;
pub fn insn_len()->usize {std::mem::size_of::<InsnT>() << 3}

pub type RegT = u64;
pub fn reg_len()->usize {std::mem::size_of::<RegT>() << 3}