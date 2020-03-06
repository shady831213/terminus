pub mod macros;

pub trait Format {
    fn rs1(&self) -> u32 {
        0
    }
    fn rs2(&self) -> u32 {
        0
    }
    fn rs3(&self) -> u32 {
        0
    }
    fn rd(&self) -> u32{
        0
    }
    fn imm(&self) -> u32{
        0
    }
    fn op(&self) -> u32{
        0
    }
}

