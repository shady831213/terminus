#[macro_export]
macro_rules! init_instruction {
    ($processor:ident, $exception:ident) => {
        pub trait Format {
            fn rs1(&self, _: &terminus_global::InsnT) -> terminus_global::InsnT {
                0
            }
            fn rs2(&self, _: &terminus_global::InsnT) -> terminus_global::InsnT {
                0
            }
            fn rd(&self, _: &terminus_global::InsnT) -> terminus_global::InsnT {
                0
            }
            fn imm(&self, _: &terminus_global::InsnT) -> terminus_global::InsnT {
                0
            }
            fn op(&self, _: &terminus_global::InsnT) -> terminus_global::InsnT {
                0
            }
            fn imm_len(&self) -> usize {
                0
            }
        }

        pub trait Execution {
            fn execute(&self, p: &mut $processor) -> Result<(), $exception>;
        }


        pub trait InstructionImp: Format + Execution + Send + Sync {}

        pub struct Instruction(Box<dyn InstructionImp>);

        impl Instruction {
            pub fn new<T: 'static + InstructionImp>(f: T) -> Instruction {
                Instruction(Box::new(f))
            }
        }

        impl std::ops::Deref for Instruction {
            type Target = Box<dyn InstructionImp>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}