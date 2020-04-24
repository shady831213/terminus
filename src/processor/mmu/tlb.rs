#[derive(Default, Copy, Clone)]
struct TLBEntry {
    valid: bool,
    vpn: u64,
    ppn: u64,
}

pub struct TLB {
    entries: [TLBEntry; 256],
    size: usize,
}

impl TLB {
    pub fn new() -> TLB {
        TLB {
            entries: [TLBEntry::default(); 256],
            size: 256,
        }
    }
    #[inline(never)]
    pub fn get_ppn(&self, vpn: u64) -> Option<u64> {
        let e = &self.entries[(vpn as usize) & (self.size - 1)];
        if e.valid && e.vpn == vpn {
            Some(e.ppn)
        } else {
            None
        }
    }
    #[inline(never)]
    pub fn set_entry(&mut self, vpn: u64, ppn: u64) {
        let e = &mut self.entries[(vpn as usize) & (self.size - 1)];
        e.valid = true;
        e.vpn = vpn;
        e.ppn = ppn;
    }


    pub fn invalid_all(&mut self) {
        self.entries.iter_mut().for_each(|e| { e.valid = false })
    }
}