#[derive(Debug,Eq, PartialEq)]
pub enum Exception {
    IllegalInsn(u32)
}