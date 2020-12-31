use crate::processor::{ProcessorState, Processor, HasCsr, NoCsr};
use crate::prelude::*;
use paste::paste;

trait HasStepCb {
    fn step_cb(&self, p: &Processor);
}

trait NoStepCb {
    fn step_cb(&self, _: &Processor) {}
}

macro_rules! declare_extension {
    ($($extension:ident),+ ) => {
        $(
           pub mod $extension;
           use $extension::*;
         )+
         paste! {
            pub enum Extension {
                $(
                    [<$extension:upper>]([<Extension $extension:upper>]),
                )+
                InvalidExtension
            }
            impl Extension {
                pub fn new(state: &ProcessorState, id: char) -> Result<Extension, String> {
                    match id.to_string().as_str() {
                        $(
                            stringify!($extension) => Ok(Extension::[<$extension:upper>]([<Extension $extension:upper>]::new(state))),
                        )+
                        _ => Err(format!("unsupported extension \'{}\', supported extension is {} !", id,  stringify!($($extension,)+)))
                    }
                }
                pub fn csr_write(&self, state: &ProcessorState, addr: InsnT, value: RegT) -> Option<()> {
                    match self {
                        $(
                            Extension::[<$extension:upper>]($extension) => $extension.csr_write(state, addr, value),
                        )+
                        _  => None
                    }
                }
                pub fn csr_read(&self, state: &ProcessorState, addr: InsnT) -> Option<RegT> {
                    match self {
                        $(
                            Extension::[<$extension:upper>]($extension) => $extension.csr_read(state, addr),
                        )+
                        _  => None
                    }
                }
                pub fn step_cb(&self, p: &Processor) {
                    match self {
                        $(
                            Extension::[<$extension:upper>]($extension) => $extension.step_cb(p),
                        )+
                        _ => {}
                    }
                }
            }
         }
    }
}

declare_extension!(a, c, d, f, i, m, s, u);