use terminus_global::InsnT;
#[derive(Debug,Eq, PartialEq)]
pub enum Exception {
    IllegalInsn(InsnT)
}