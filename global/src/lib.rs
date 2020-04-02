pub type InsnT = u32;

pub fn insn_len() -> usize { std::mem::size_of::<InsnT>() << 3 }

pub type RegT = u64;

pub type SRegT = i64;

pub fn reg_len() -> usize { std::mem::size_of::<RegT>() << 3 }

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum XLen {
    X32,
    X64,
}

impl XLen {
    pub fn len(&self) -> usize {
        match self {
            XLen::X32 => 32,
            XLen::X64 => 64
        }
    }
}

pub fn sext(value: RegT, len: usize) -> RegT {
    let bit_len = std::mem::size_of::<RegT>() << 3;
    assert!(len > 0 && len <= bit_len);
    if len == bit_len {
        return value;
    }
    let sign = value >> (len - 1) as RegT & 0x1;
    let mask = ((1 as RegT) << (len as RegT)) - 1 as RegT;
    if sign == 0 {
        value & mask
    } else {
        let high = ((1 as RegT) << (bit_len as RegT - len as RegT) - 1 as RegT) << (len as RegT);
        value & mask | high
    }
}
