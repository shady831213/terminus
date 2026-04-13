use gdbstub::arch::RegId;
use std::num::NonZeroUsize;

/// Valid CSR address range per RISC-V spec: 0x000-0xFFF
const CSR_ADDR_MIN: u16 = 0x000;
const CSR_ADDR_MAX: u16 = 0xFFF;

/// Canonical CSR map: regnum offset (0-25) -> CSR address.
/// These are the CSRs that GDB can access by register number.
const CSR_MAP: [u16; 26] = [
    0x100, 0x104, 0x105, 0x140, 0x141, 0x142, 0x143, 0x144, 0x180, 0x300, 0x301, 0x302, 0x303,
    0x304, 0x305, 0x340, 0x341, 0x342, 0x343, 0x344, 0xB00, 0xB02, 0xF11, 0xF12, 0xF13, 0xF14,
];

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

/// Convert a CSR register number index (0-25) to a CSR address.
/// Returns 0 for out-of-range indices (should not happen via from_raw_id).
fn csr_regnum_to_addr(csr_idx: u16) -> u16 {
    CSR_MAP.get(csr_idx as usize).copied().unwrap_or(0)
}

/// Convert a CSR address to a GDB register number.
/// Returns None if the address is out of the valid CSR range (0x000-0xFFF)
/// or if the address is not in the supported CSR map.
pub fn csr_addr_to_regnum(csr_addr: u16) -> Option<usize> {
    if csr_addr < CSR_ADDR_MIN || csr_addr > CSR_ADDR_MAX {
        return None;
    }
    CSR_MAP
        .iter()
        .position(|&addr| addr == csr_addr)
        .map(|pos| pos + 66)
}

/// Check if a CSR address is within the valid RISC-V CSR address range (0x000-0xFFF).
pub fn is_valid_csr_addr(addr: u16) -> bool {
    addr >= CSR_ADDR_MIN && addr <= CSR_ADDR_MAX
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

    #[test]
    fn test_csr_bounds_checking_valid_range() {
        assert!(is_valid_csr_addr(0x000));
        assert!(is_valid_csr_addr(0x100));
        assert!(is_valid_csr_addr(0x300));
        assert!(is_valid_csr_addr(0xFFF));
    }

    #[test]
    fn test_csr_bounds_checking_invalid_range() {
        assert!(!is_valid_csr_addr(0x1000));
        assert!(!is_valid_csr_addr(0xFFFF));
    }

    #[test]
    fn test_csr_addr_to_regnum_out_of_range() {
        assert_eq!(csr_addr_to_regnum(0x1000), None);
        assert_eq!(csr_addr_to_regnum(0xFFFF), None);
    }

    #[test]
    fn test_csr_mapping_roundtrip() {
        for (idx, &expected_addr) in CSR_MAP.iter().enumerate() {
            let regnum = csr_addr_to_regnum(expected_addr);
            assert_eq!(
                regnum,
                Some(idx + 66),
                "CSR addr 0x{:03X} should map to regnum {}",
                expected_addr,
                idx + 66
            );

            let back_addr = csr_regnum_to_addr(idx as u16);
            assert_eq!(
                back_addr, expected_addr,
                "Regnum offset {} should map back to CSR addr 0x{:03X}",
                idx, expected_addr
            );
        }
    }

    #[test]
    fn test_csr_map_all_within_bounds() {
        for &addr in &CSR_MAP {
            assert!(
                is_valid_csr_addr(addr),
                "CSR addr 0x{:03X} should be within valid range",
                addr
            );
        }
    }

    #[test]
    fn test_csr_map_sorted_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for &addr in &CSR_MAP {
            assert!(
                seen.insert(addr),
                "Duplicate CSR addr 0x{:03X} in CSR_MAP",
                addr
            );
        }
    }
}
