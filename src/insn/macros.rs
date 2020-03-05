#[macro_export(local_inner_macros)]
macro_rules! instruction {
    ($name:ident; $format:tt; $code:expr; $($rest:tt)*) => {
        pub struct $name(u32);
        bitfield_bitrange!{struct $name(u32)}
        __instruction_format!($name; $format;)
    };
}


#[macro_export]
macro_rules! __instruction_format {
    //user defined
    ($name:ident; USER_DEFINE;) => {
    };
    //I
    ($name:ident; R;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:31, 0;
                op,_:6,0;
                rd,_:11, 7;
                rs1,_:19, 15;
                rs2,_:24, 20;
             }
        }
    };
    ($name:ident; I;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:31, 0;
                op,_:6,0;
                rd,_:11, 7;
                rs1,_:19, 15;
                imm,_:31:20;
             }
        }
    };
    ($name:ident; S;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:31, 0;
                op,_:6,0;
                rs1,_:19, 15;
                rs2,_:24, 20;
             }
        }
        fn imm(&self)->u32 {
            self.bit_range(31,25) << 5 | self.bit_range(11, 7)
        }
    };
    ($name:ident; B;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:31, 0;
                op,_:6,0;
                rs1,_:19, 15;
                rs2,_:24, 20;
             }
        }
        fn imm(&self)->u32 {
             self.bit_range(31, 31) << 12 | self.bit_range(7, 7) << 11 | self.bit_range(30,25) << 5 | self.bit_range(11, 8) << 1
        }
    };
    ($name:ident; U;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:31, 0;
                op,_:6,0;
                rd,_:11, 7;
                imm,_:31,12;
             }
        }
    };
    ($name:ident; J;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:31, 0;
                op,_:6,0;
                rd,_:11, 7;
             }
        }
        fn imm(&self)->u32 {
             self.bit_range(31, 31) << 20 | self.bit_range(19, 12) << 12 | self.bit_range(20,20) << 11 | self.bit_range(30, 21) << 1
        }
    };
    //compress format
    ($name:ident; CR;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:15, 0;
                op,_:1,0;
                rs2,_:6,2;
                rs1,_:11, 7;
                rd,_:11, 7;
             }
        }
    };
    ($name:ident; CI;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:15, 0;
                op,_:1,0;
                rs1,_:11, 7;
                rd,_:11, 7;
             }
             fn imm(&self)->u32 {
                self.bit_range(12,12) << 5 | self.bit_range(6, 2)
             }
        }
    };
    ($name:ident; CSS;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:15, 0;
                op,_:1,0;
                rs2,_:6,2;
                imm,_:12, 7;
             }
        }
    };
    ($name:ident; CIW;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:15, 0;
                op,_:1,0;
                rd,_:4, 2;
                imm,_:12, 5;
             }
        }
    };
    ($name:ident; CL;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:15, 0;
                op,_:1,0;
                rd,_:4, 2;
                rs1,_:9, 7;
             }
             fn imm(&self)->u32 {
                self.bit_range(12,10) << 2 | self.bit_range(6, 5)
             }
        }
    };
    ($name:ident; CS;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:15, 0;
                op,_:1,0;
                rs2,_:4, 2;
                rs1,_:9, 7;
             }
             fn imm(&self)->u32 {
                self.bit_range(12,10) << 2 | self.bit_range(6, 5)
             }
        }
    };
    ($name:ident; CB;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:15, 0;
                op,_:1,0;
                rs1,_:9, 7;
             }
             fn imm(&self)->u32 {
                self.bit_range(12,10) << 6 | self.bit_range(6, 1)
             }
        }
    };
    ($name:ident; CJ;) => {
        impl Format for $name {
             bitfield_fields!{
                u32;
                ir,_:15, 0;
                op,_:1,0;
                imm,_:12, 2;
             }
        }
    };
}