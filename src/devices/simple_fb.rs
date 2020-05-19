use std::cell::RefCell;
use terminus_spaceport::devices::{FrameBuffer, Display};
use std::cmp::min;
use terminus_spaceport::memory::prelude::*;

const SIMPLE_FB_PAGE_SIZE: u32 = 4096;
const SIMPLE_FB_PAGE_SIZE_SHIFT: u32 = 12;
const SIMPLE_FB_MERGE_TH: u32 = 3;
const SIMPLE_FB_REFRESH_BATCH: u32 = 32;
const SIMPLE_FB_REFRESH_BATCH_SHIFT: u32 = 5;

#[derive_io(Bytes, U8)]
pub struct SimpleFb {
    fb: RefCell<Vec<u8>>,
    pages: u32,
    dirties: RefCell<Vec<u32>>,
}

impl SimpleFb {
    pub fn new<D: Display>(d: &D) -> SimpleFb {
        let size = d.width() * d.height() * 4;
        let pages = (size + SIMPLE_FB_PAGE_SIZE as usize - 1) / (SIMPLE_FB_PAGE_SIZE as usize);
        let dirties_len = (pages + SIMPLE_FB_REFRESH_BATCH as usize - 1) >> (SIMPLE_FB_REFRESH_BATCH_SHIFT as usize);
        SimpleFb {
            fb: RefCell::new(vec![0; size]),
            pages:pages as u32,
            dirties: RefCell::new(vec![0; dirties_len]),
        }
    }
}

impl FrameBuffer for SimpleFb {
    fn refresh<D: Display>(&self, d: &D) -> Result<(), String> {
        let mut data = self.fb.borrow_mut();
        let mut dirties_ref = self.dirties.borrow_mut();
        let mut page_idx: u32 = 0;
        let mut y_start: u32 = 0;
        let mut y_end: u32 = 0;
        let width = d.width() as u32;
        let stride = width << 2;
        let height = d.height() as u32;
        while page_idx < self.pages {
            let dirties_offset = page_idx >> SIMPLE_FB_REFRESH_BATCH_SHIFT;
            let dirties = dirties_ref[dirties_offset as usize];
            if dirties != 0 {
                let mut page_offset: u32 = 0;
                while dirties != 0 {
                    while (dirties >> page_offset) & 0x1 == 0 {
                        page_offset += 1;
                    }
                    let byte_offset = (page_idx + page_offset) >> SIMPLE_FB_PAGE_SIZE_SHIFT;
                    let y_start_offset = byte_offset / stride;
                    let y_end_offset = min(((byte_offset + SIMPLE_FB_PAGE_SIZE - 1) / stride) + 1, height);
                    if y_start == y_end {
                        y_start = y_end_offset;
                        y_end = y_end_offset;
                    } else if y_start_offset <= y_end + SIMPLE_FB_MERGE_TH {
                        y_end = y_end_offset
                    } else {
                        let byte_start = (y_start * stride) as usize;
                        let byte_end = (y_end * stride) as usize;
                        d.draw(&mut data[byte_start..byte_end], 0, y_start as i32, width, y_end - y_start)?
                    }
                }
                dirties_ref[dirties_offset as usize] = 0;
            }
            page_idx += SIMPLE_FB_REFRESH_BATCH
        }

        if y_start != y_end {
            let byte_start = (y_start * stride) as usize;
            let byte_end = (y_end * stride) as usize;
            d.draw(&mut data[byte_start..byte_end], 0, y_start as i32, width, y_end - y_start)?
        }
        Ok(())
    }
}

impl BytesAccess for SimpleFb {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        let offset = *addr as  usize;
        self.fb.borrow_mut()[offset..offset + data.len()].copy_from_slice(data);
        Ok(data.len())
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        let offset = *addr as  usize;
        data.copy_from_slice(&self.fb.borrow()[offset..offset + data.len()]);
        Ok(data.len())
    }
}

impl U8Access for SimpleFb {
    fn write(&self, addr: &u64, data: u8) {
        (*self.fb.borrow_mut())[*addr as usize] = data
    }

    fn read(&self, addr: &u64) -> u8 {
        (*self.fb.borrow())[*addr as usize]
    }
}