#[macro_export(local_inner_macros)]
macro_rules! init_decoder {
    ($inst:ty) => {
        use crate::linkme::*;
        #[derive(Debug)]
        pub enum Error {
            Illegal($inst),
        }

        impl Error {
            pub fn ir(&self) -> $inst {
                match self {
                    Error::Illegal(ir) => *ir,
                }
            }
        }

        pub trait Decoder:Send+Sync {
            fn code(&self) -> $inst;
            fn mask(&self) -> $inst;
            fn matched(&self, ir: &$inst) -> bool;
            fn decode(&self) -> &Instruction;
            fn name(&self) -> String;
        }

        pub trait InsnMap {
            fn registery<T: 'static + Decoder>(&mut self, decoder: T);
            fn decode(&self, ir: &$inst) -> Result<&Instruction, Error>;
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