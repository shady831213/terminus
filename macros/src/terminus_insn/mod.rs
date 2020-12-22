mod init_decoder;
mod init_instruction;
mod init_treemap;
mod init_simplemap;
#[macro_export]
macro_rules! terminus_insn {
    ($inst:ty, $processor:ident, $exception:ident) => {
        terminus_insn!($inst, $processor, $exception, Tree);
    };
    ($inst:ty, $processor:ident, $exception:ident, USER_DEFINE) => {
        terminus_insn!(@common $inst, $processor, $exception);
    };
    ($inst:ty, $processor:ident, $exception:ident, Tree) => {
        terminus_insn!(@common $inst, $processor, $exception);
        init_treemap!($inst);
    };
    ($inst:ty, $processor:ident, $exception:ident, Simple) => {
        terminus_insn!(@common $inst, $processor, $exception);
        init_simplemap!($inst);
    };
    (@common $inst:ty, $processor:ident, $exception:ident) => {
        init_instruction!($processor, $exception, $inst);
        init_decoder!($inst);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __terminus_insn_unreachable {
    () => {
        unreachable!()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __terminus_insn_panic {
    ($($s:tt)*) => {
        panic!($($s)*)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __terminus_insn_format {
    ($($s:tt)*) => {
        format!($($s)*)
    };
}