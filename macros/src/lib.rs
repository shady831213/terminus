#[macro_use]
pub extern crate bitfield;

pub use bitfield::*;

pub trait InsnCoding {
    fn ir(&self) -> u32;
    fn code(&self) -> u32;
    fn mask(&self) -> u32;
}