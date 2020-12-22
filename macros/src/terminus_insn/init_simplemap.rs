#[macro_export(local_inner_macros)]
macro_rules! init_simplemap {
    ($inst:ty) => {
        pub type GlobalInsnMap = SimpleInsnMap;
        pub struct SimpleInsnMap(std::collections::HashMap<InsnT, Box<dyn Decoder>>);

        impl SimpleInsnMap {
            pub fn new() -> SimpleInsnMap {
                SimpleInsnMap(std::collections::HashMap::new())
            }
        }

        impl InsnMap for SimpleInsnMap {
            fn registery<T: 'static + Decoder>(&mut self, decoder: T) {
                self.0.insert(decoder.code(), Box::new(decoder));
            }

            fn decode(&self, ir: &$inst) -> Result<&Instruction, Error> {
                let decoder = self.0.values().find(|d| { d.matched(ir) });
                if let Some(d) = decoder {
                    Ok(d.decode())
                } else {
                    Err(Error::Illegal(*ir))
                }
            }
        }
    };
}