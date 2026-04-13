use gdbstub::arch::RegId;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiscvRegId {
    Gpr(u16),
    Pc,
    Fpr(u16),
    Fcsr,
    Csr(u16),
    Priv,
}

impl RegId for RiscvRegId {
    fn from_raw_id(id: usize) -> Option<(Self, Option<NonZeroUsize>)> {
        match id {
            0..=31 => Some((RiscvRegId::Gpr(id as u16), NonZeroUsize::new(1))),
            32 => Some((RiscvRegId::Pc, NonZeroUsize::new(1))),
            33..=64 => Some((RiscvRegId::Fpr((id - 33) as u16), NonZeroUsize::new(1))),
            65 => Some((RiscvRegId::Fcsr, NonZeroUsize::new(1))),
            66..=91 => Some((
                RiscvRegId::Csr(csr_regnum_to_addr((id - 66) as u16)),
                NonZeroUsize::new(1),
            )),
            132 => Some((RiscvRegId::Priv, None)),
            _ => None,
        }
    }
}

fn csr_regnum_to_addr(csr_idx: u16) -> u16 {
    const CSR_MAP: [u16; 26] = [
        0x100, 0x104, 0x105, 0x140, 0x141, 0x142, 0x143, 0x144, 0x180, 0x300, 0x301, 0x302, 0x303,
        0x304, 0x305, 0x340, 0x341, 0x342, 0x343, 0x344, 0xB00, 0xB02, 0xF11, 0xF12, 0xF13, 0xF14,
    ];
    CSR_MAP[csr_idx as usize]
}

pub fn csr_addr_to_regnum(csr_addr: u16) -> Option<usize> {
    const CSR_REVERSE_MAP: &[u16] = &[
        0x100, 0x104, 0x105, 0x140, 0x141, 0x142, 0x143, 0x144, 0x180, 0x300, 0x301, 0x302, 0x303,
        0x304, 0x305, 0x340, 0x341, 0x342, 0x343, 0x344, 0xB00, 0xB02, 0xF11, 0xF12, 0xF13, 0xF14,
    ];
    CSR_REVERSE_MAP
        .iter()
        .position(|&addr| addr == csr_addr)
        .map(|pos| pos + 66)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg_id(id: usize) -> Option<RiscvRegId> {
        RiscvRegId::from_raw_id(id).map(|(r, _)| r)
    }

    #[test]
    fn test_gpr_mapping() {
        assert_eq!(reg_id(0), Some(RiscvRegId::Gpr(0)));
        assert_eq!(reg_id(31), Some(RiscvRegId::Gpr(31)));
    }

    #[test]
    fn test_pc_mapping() {
        assert_eq!(reg_id(32), Some(RiscvRegId::Pc));
    }

    #[test]
    fn test_fpr_mapping() {
        assert_eq!(reg_id(33), Some(RiscvRegId::Fpr(0)));
        assert_eq!(reg_id(64), Some(RiscvRegId::Fpr(31)));
    }

    #[test]
    fn test_fcsr_mapping() {
        assert_eq!(reg_id(65), Some(RiscvRegId::Fcsr));
    }

    #[test]
    fn test_csr_mapping() {
        assert_eq!(reg_id(66), Some(RiscvRegId::Csr(0x100)));
        assert_eq!(reg_id(91), Some(RiscvRegId::Csr(0xF14)));
    }

    #[test]
    fn test_priv_mapping() {
        assert_eq!(reg_id(132), Some(RiscvRegId::Priv));
    }

    #[test]
    fn test_invalid_ids() {
        assert_eq!(reg_id(92), None);
        assert_eq!(reg_id(133), None);
        assert_eq!(reg_id(9999), None);
    }

    #[test]
    fn test_csr_addr_to_regnum() {
        assert_eq!(csr_addr_to_regnum(0x100), Some(66));
        assert_eq!(csr_addr_to_regnum(0xF14), Some(91));
        assert_eq!(csr_addr_to_regnum(0x123), None);
    }
}
