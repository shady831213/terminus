#[macro_export]
macro_rules! init_decoder {
    () => {
        use crate::linkme::*;
        #[derive(Debug)]
        pub enum Error {
            Illegal(terminus_global::InsnT),
        }

        impl Error {
            pub fn ir(&self) -> terminus_global::InsnT {
                match self {
                    Error::Illegal(ir) => *ir,
                }
            }
        }

        pub trait Decoder:Send+Sync {
            fn code(&self) -> terminus_global::InsnT;
            fn mask(&self) -> terminus_global::InsnT;
            fn matched(&self, ir: &terminus_global::InsnT) -> bool;
            fn decode(&self) -> &Instruction;
            fn name(&self) -> String;
        }

        pub trait InsnMap {
            fn registery<T: 'static + Decoder>(&mut self, decoder: T);
            fn decode(&self, ir: &terminus_global::InsnT) -> Result<&Instruction, Error>;
            fn lock(&mut self) {}
        }

        lazy_static! {
            pub static ref GDECODER:GlobalInsnMap = {
                let mut map = GlobalInsnMap::new();
                for r in REGISTERY_INSN {
                    r(&mut map)
                }
                map.lock();
                map
            };
        }

        #[distributed_slice]
        pub static REGISTERY_INSN: [fn(&mut GlobalInsnMap)] = [..];
    };
}