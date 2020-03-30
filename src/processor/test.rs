use super::*;
use terminus_macros::*;
use terminus_global::*;

#[test]
fn pmp_basic_test() {
    let mut p = Processor::new(XLen::X32);
    //no valid region
    assert_eq!(p.mmu().match_pmpcfg_entry(0, |p, entry| { true }), None);
    //NA4
    p.basic_csr.pmpcfg0.set_bit_range(4, 3, PmpAType::NA4.into());
    p.basic_csr.pmpaddr0.set(0x8000_0000 >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x8000_0000, |p, entry| { true }).is_some());
    //NAPOT
    p.basic_csr.pmpcfg3.set_bit_range(4, 3, PmpAType::NAPOT.into());
    p.basic_csr.pmpaddr12.set((0x2000_0000 + 0x1_0000 - 1) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x2000_0000, |p, entry| { true }).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2000_ffff, |p, entry| { true }).is_some());
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2000_ffff, |p, entry| { true }), p.mmu().match_pmpcfg_entry(0x2000_0000, |p, entry| { true }));
    assert_eq!(p.mmu().match_pmpcfg_entry(0x1000_ffff, |p, entry| { true }), None);
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2001_0000, |p, entry| { true }), None);
    //TOR
    p.basic_csr.pmpcfg3.set_bit_range(12, 11, PmpAType::TOR.into());
    p.basic_csr.pmpaddr13.set((0x2000_0000 + 0x1_0000) >> 2);
    p.basic_csr.pmpcfg3.set_bit_range(20, 19, PmpAType::TOR.into());
    p.basic_csr.pmpaddr14.set((0x2000_0000 + 0x2_0000) >> 2);
    assert!(p.mmu().match_pmpcfg_entry(0x2001_0000, |p, entry| { true }).is_some());
    assert!(p.mmu().match_pmpcfg_entry(0x2001_ffff, |p, entry| { true }).is_some());
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2002_0000, |p, entry| { true }), None);
    assert_eq!(p.mmu().match_pmpcfg_entry(0x2001_0000, |p, entry| { entry.l() == 1 }), None);
    p.basic_csr.pmpcfg3.set_bit_range(23, 23, 1);
    assert!(p.mmu().match_pmpcfg_entry(0x2001_0000, |p, entry| { entry.l() == 1 }).is_some());

}