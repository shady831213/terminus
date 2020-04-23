#[derive(Default, Copy, Clone)]
struct TLBEntry {
    vpn: u64,
    ppn: u64,
}

pub struct TLB {
    entries: Vec<Option<TLBEntry>>,
    size: usize,
}

impl TLB {
    pub fn new(size: usize) -> TLB {
        assert!(size.is_power_of_two());
        TLB {
            entries: vec![None; size],
            size,
        }
    }

    pub fn get_ppn(&mut self, vpn: u64) -> Option<u64> {
        if let Some(ref entry) = self.entries[(vpn as usize) & (self.size - 1)] {
            if entry.vpn == vpn {
                Some(entry.ppn)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_entry(&mut self, vpn: u64, ppn: u64) {
        self.entries[(vpn as usize) & (self.size - 1)] = Some(TLBEntry{vpn, ppn})
    }

    // pub fn invalid(&mut self, vpn: u64) {
    //     self.entries.retain(|e|{e.vpn != vpn})
    // }

    pub fn invalid_all(&mut self) {
        self.entries.iter_mut().for_each(|e|{*e = None})
    }

}