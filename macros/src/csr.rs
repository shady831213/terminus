#[macro_export]
macro_rules! decl_csr {
    ($(#[$attribute:meta])* pub struct $($rest:tt)*) => {
        decl_csr!($(#[$attribute])* (pub) struct $($rest)*);
    };
    ($(#[$attribute:meta])* struct $($rest:tt)*) => {
        decl_csr!($(#[$attribute])* () struct $($rest)*);
    };
    ($(#[$attribute:meta])* ($($vis:tt)*) struct $name:ident; $($rest:tt)*) => {
        bitfield!($(#[$attribute])* ($($vis)*) struct $name(RegT);impl Debug; $($rest)*);
        impl $name {
            pub fn new(init:RegT) -> $name {
                $name(init)
            }
        }
    };
}