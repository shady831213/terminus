use std::cell::RefCell;
use terminus_spaceport::devices::{FrameBuffer, Display, KeyBoard, Mouse};
use std::cmp::min;
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;

const SIMPLE_FB_PAGE_SIZE: u32 = 4096;
const SIMPLE_FB_PAGE_SIZE_SHIFT: u32 = 12;
const SIMPLE_FB_MERGE_TH: u32 = 3;
const SIMPLE_FB_REFRESH_BATCH: u32 = 32;
const SIMPLE_FB_REFRESH_BATCH_SHIFT: u32 = 5;

pub struct Fb {
    fb: RefCell<Vec<u8>>,
    pages: u32,
    dirties: RefCell<Vec<u32>>,
}

impl Fb {
    pub fn new<D: Display>(d: &D) -> Fb {
        let size = d.width() * d.height() * 4;
        let pages = (size + SIMPLE_FB_PAGE_SIZE as usize - 1) / (SIMPLE_FB_PAGE_SIZE as usize);
        let dirties_len = (pages + SIMPLE_FB_REFRESH_BATCH as usize - 1) >> (SIMPLE_FB_REFRESH_BATCH_SHIFT as usize);
        Fb {
            fb: RefCell::new(vec![0; size]),
            pages: pages as u32,
            dirties: RefCell::new(vec![0; dirties_len]),
        }
    }
    pub fn size(&self) -> u32 {
        self.pages << SIMPLE_FB_PAGE_SIZE_SHIFT
    }
    fn set_dirty(&self, offset:&u64) {
        let page = ((*offset) as u32) >> SIMPLE_FB_PAGE_SIZE_SHIFT;
        let bits = page & ((1 << SIMPLE_FB_REFRESH_BATCH_SHIFT) -1);
        let pos = page >> SIMPLE_FB_REFRESH_BATCH_SHIFT;
        self.dirties.borrow_mut()[pos as usize] |= 1 << bits
    }
}

impl FrameBuffer for Fb {
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
            let mut dirties = dirties_ref[dirties_offset as usize];
            if dirties != 0 {
                let mut page_offset: u32 = 0;
                while dirties != 0{
                    while (dirties >> page_offset) & 0x1 == 0 {
                        page_offset += 1;
                    }
                    dirties &= !(1 << page_offset);
                    let byte_offset = (page_idx + page_offset) << SIMPLE_FB_PAGE_SIZE_SHIFT;
                    let y_start_offset = byte_offset / stride;
                    let y_end_offset = min(((byte_offset + SIMPLE_FB_PAGE_SIZE - 1) / stride) + 1, height);
                    if y_start == y_end {
                        y_start = y_start_offset;
                        y_end = y_end_offset;
                    } else if y_start_offset <= y_end + SIMPLE_FB_MERGE_TH {
                        y_end = y_end_offset
                    } else {
                        let byte_start = (y_start * stride) as usize;
                        let byte_end = (y_end * stride) as usize;
                        d.draw(&mut data[byte_start..byte_end], 0, y_start as i32, width, y_end - y_start)?;
                        y_start = y_start_offset;
                        y_end = y_end_offset;
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

#[derive_io(Bytes, U8)]
pub struct SimpleFb(Rc<Fb>);

impl SimpleFb {
    pub fn new(fb: &Rc<Fb>) -> SimpleFb {
        SimpleFb(fb.clone())
    }
}

impl BytesAccess for SimpleFb {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        self.0.set_dirty(addr);
        let offset = *addr as usize;
        self.0.fb.borrow_mut()[offset..offset + data.len()].copy_from_slice(data);
        Ok(data.len())
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        let offset = *addr as usize;
        data.copy_from_slice(&self.0.fb.borrow()[offset..offset + data.len()]);
        Ok(data.len())
    }
}

impl U8Access for SimpleFb {
    fn write(&self, addr: &u64, data: u8) {
        self.0.set_dirty(addr);
        (*self.0.fb.borrow_mut())[*addr as usize] = data
    }

    fn read(&self, addr: &u64) -> u8 {
        (*self.0.fb.borrow())[*addr as usize]
    }
}

//fixme:should be remove
pub struct DummyKb {}

impl KeyBoard for DummyKb {
    fn send_key_event(&self, key_down: bool, val: u32) {}
}

pub struct DummyMouse {}

impl Mouse for DummyMouse {
    fn send_mouse_event(&self, x: i32, y: i32, z: i32, buttons: u32) {}
    fn mouse_absolute(&self) -> bool { false }
}