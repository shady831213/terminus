#[macro_export]
macro_rules! decl_csr {
    ($(#[$attribute:meta])* pub struct $($rest:tt)*) => {
        decl_csr!($(#[$attribute])* (pub) struct $($rest)*);
    };
    ($(#[$attribute:meta])* struct $($rest:tt)*) => {
        decl_csr!($(#[$attribute])* () struct $($rest)*);
    };
    ($(#[$attribute:meta])* ($($vis:tt)*) struct $name:ident($t:ty); $($rest:tt)*) => {
        bitfield!($(#[$attribute])* ($($vis)*) struct $name($t); $($rest)*);
        impl $name {
            fn write(&mut self, value:$t) {
                self.0 = value
            }
            fn read(&self) -> $t {
                self.0
            }
        }
        impl Deref for  $name {
            type Target = $t;
            fn deref(&self) -> &Self::Target {
            &self.0
            }
        }

        impl DerefMut for  $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
            }
        }
    };
}