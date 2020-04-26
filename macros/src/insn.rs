#[macro_export]
macro_rules! insn_format {
    //user defined
    ($name:ident, USER_DEFINE) => {
    };
    //I
    ($name:ident, R) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x7f
             }
             fn rd(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x1f
             }
             fn rs1(&self,code:InsnT)->InsnT {
                (code >> 15) & 0x1f
             }
             fn rs2(&self,code:InsnT)->InsnT {
                (code >> 20) & 0x1f
             }
        }
    };
    ($name:ident, I) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x7f
             }
             fn rd(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x1f
             }
             fn rs1(&self,code:InsnT)->InsnT {
                (code >> 15) & 0x1f
             }
             fn imm(&self,code:InsnT)->InsnT {
                (code >> 20) & 0xfff
             }
             fn imm_len(&self)-> usize {
                12
             }
        }
    };
    ($name:ident, S) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x7f
             }
             fn rs1(&self,code:InsnT)->InsnT {
                (code >> 15) & 0x1f
             }
             fn rs2(&self,code:InsnT)->InsnT {
                (code >> 20) & 0x1f
             }
             fn imm(&self,code:InsnT)->InsnT {
                ((code >> 7) & 0x1f) |  ((code >> 25) & 0x7f) << 5
             }

            fn imm_len(&self)-> usize {
                12
            }
            
            
        }
    };
    ($name:ident, B) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x7f
             }
             fn rs2(&self,code:InsnT)->InsnT {
                (code >> 20) & 0x1f
             }
             fn rs1(&self,code:InsnT)->InsnT {
                (code >> 15) & 0x1f
             }
             fn imm(&self,code:InsnT)->InsnT {
                ((code >> 31) & 0x1) << 12 | ((code >> 7) & 0x1) << 11 | ((code >> 25) & 0x3f) << 5 | ((code >> 8) & 0xf) << 1
             }
             fn imm_len(&self)-> usize {
                13
             }
        }
    };
    ($name:ident, U) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x7f
             }
             fn rd(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x1f
             }
             fn imm(&self,code:InsnT)->InsnT {
                (code >> 12) << 12
             }
             fn imm_len(&self)-> usize {
                32
             }
             
             
        }
    };
    ($name:ident, J) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x7f
             }
             fn rd(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x1f
             }
             fn imm(&self,code:InsnT)->InsnT {
                ((code >> 31) & 0x1) << 20 | ((code >> 12) & 0xff) << 12 | ((code >> 20) & 0x1) << 11 | ((code >> 21) & 0x3ff) << 1
             }

             fn imm_len(&self)-> usize {
                21
             }
            
            
        }
    };
    //compress format
    ($name:ident, CR) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn rd(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x1f
             }
             fn rs1(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x1f
             }
             fn rs2(&self,code:InsnT)->InsnT {
                (code >> 2) & 0x1f
             }
        }
    };
    ($name:ident, CIW) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn rd(&self,code:InsnT)->InsnT {
                ((code >> 2) & 0x7) + 8
             }
             fn imm(&self,code:InsnT)->InsnT {
                (code >> 5) & 0xff
             }
             fn imm_len(&self)-> usize {
                8
             }
        }
    };
    ($name:ident, CI) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn rd(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x1f
             }
             fn rs1(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x1f
             }
             fn imm(&self,code:InsnT)->InsnT {
                ((code >> 12) & 0x1) << 5 | (code >> 2) & 0x1f
             }

             fn imm_len(&self)-> usize {
                6
             }
             
             
        }
    };
    ($name:ident, CSS) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn rs2(&self,code:InsnT)->InsnT {
                (code >> 2) & 0x1f
             }
             fn imm(&self,code:InsnT)->InsnT {
                (code >> 7) & 0x3f
             }
             fn imm_len(&self)-> usize {
                6
             }
        }
    };
    ($name:ident, CL) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn rd(&self,code:InsnT)->InsnT {
                ((code >> 2) & 0x7) + 8
             }
             fn rs1(&self,code:InsnT)->InsnT {
                ((code >> 7) & 0x7) + 8
             }
             fn imm(&self,code:InsnT)->InsnT {
                ((code >> 10) & 0x7) << 2 | (code >> 5) & 0x3
             }

             fn imm_len(&self)-> usize {
                5
             }
        }
    };
    ($name:ident, CS) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn rs2(&self,code:InsnT)->InsnT {
                ((code >> 2) & 0x7) + 8
             }
             fn rs1(&self,code:InsnT)->InsnT {
                ((code >> 7) & 0x7) + 8
             }
             fn imm(&self,code:InsnT)->InsnT {
                ((code >> 10) & 0x7) << 2 | (code >> 5) & 0x3
             }
             fn imm_len(&self)-> usize {
                5
             }
        }
    };
    ($name:ident, CB) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn rd(&self,code:InsnT)->InsnT {
                ((code >> 7) & 0x7) + 8
             }
             fn rs1(&self,code:InsnT)->InsnT {
                ((code >> 7) & 0x7) + 8
             }
             fn imm(&self,code:InsnT)->InsnT {
                ((code >> 10) & 0x7) << 5 | (code >> 2) & 0x1f
             }
             fn imm_len(&self)-> usize {
                8
             }
        }
    };
    ($name:ident, CA) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn rd(&self,code:InsnT)->InsnT {
                ((code >> 7) & 0x7) + 8
             }
             fn rs2(&self,code:InsnT)->InsnT {
                ((code >> 2) & 0x7) + 8
             }
             fn rs1(&self,code:InsnT)->InsnT {
                ((code >> 7) & 0x7) + 8
             }
        }
    };
    ($name:ident, CJ) => {
        impl Format for $name {
             fn op(&self,code:InsnT)->InsnT {
                code & 0x3
             }
             fn imm(&self,code:InsnT)->InsnT {
                (code >> 2) & 0x7ff
             }
             fn imm_len(&self)-> usize {
                11
             }
        }
    };
    ($name:ident, $($t:tt)*) => {
        Invalid_Format_Type!
    };
}