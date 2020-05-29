use std::sync::Mutex;
use crate::system::System;
use std::borrow::BorrowMut;

static SYS: Mutex<Option<System>> = Mutex::new(None);

pub fn init(elf_file: &str, max_int_src: usize) {
    *SYS.lock().unwrap() = Some(System::new("cosim_sys", elf_file, 10000000, max_int_src))
}