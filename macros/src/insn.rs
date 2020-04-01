#[macro_export]
macro_rules! insn_format {
    //user defined
    ($name:ident, USER_DEFINE) => {
    };
    //I
    ($name:ident, R) => {
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:6,0;
                rd,_:11, 7;
                rs1,_:19, 15;
                rs2,_:24, 20;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, I) => {
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:6,0;
                rd,_:11, 7;
                rs1,_:19, 15;
                imm,_:31, 20;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, S) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _imm1, _:31, 25;
                _imm2, _:11,7;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:6,0;
                rs1,_:19, 15;
                rs2,_:24, 20;
             }
            fn imm(&self)->InsnT {
                self._imm1() << 5 | self._imm2()
            }
            fn ir(&self)->InsnT {
                self._ir()
            }
        }
    };
    ($name:ident, B) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _imm1, _:31, 31;
                _imm2, _:7,7;
                _imm3, _:30,25;
                _imm4,_:11,8;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:6,0;
                rs1,_:19, 15;
                rs2,_:24, 20;
             }
            fn imm(&self)->InsnT {
                self._imm1() << 12 |  self._imm2() << 11 | self._imm3() << 5 | self._imm4() << 1
            }
            fn ir(&self)->InsnT {
               self._ir()
            }
        }
    };
    ($name:ident, U) => {
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:6,0;
                rd,_:11, 7;
                imm,_:31,12;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, J) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _imm1, _:31, 31;
                _imm2, _:19,12;
                _imm3, _:20,20;
                _imm4,_:30,21;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:6,0;
                rd,_:11, 7;
             }
            fn imm(&self)->InsnT {
                 self._imm1() << 20 | self._imm2()  << 12 | self._imm3()  << 11 | self._imm4()  << 1
            }
            fn ir(&self)->InsnT {
                self._ir()
            }
        }
    };
    //compress format
    ($name:ident, CR) => {
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                rs2,_:6,2;
                rs1,_:11, 7;
                rd,_:11, 7;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, CIW) => {
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                rd,_:4, 2;
                imm,_:12, 5;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, CI) => {
       impl $name {
             bitfield_fields!{
                InsnT;
                _imm1, _:12, 12;
                _imm2, _:6,2;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                rs1,_:11, 7;
                rd,_:11, 7;
             }
             fn imm(&self)->InsnT {
                self._imm1() << 5 | self._imm2()
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, CSS) => {
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                rs2,_:6,2;
                imm,_:12, 7;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, CL) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _imm1, _:12, 10;
                _imm2, _:6,5;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                rd,_:4, 2;
                rs1,_:9, 7;
             }
             fn imm(&self)->InsnT {
                self._imm1() << 2 | self._imm2()
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, CS) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _imm1, _:12, 10;
                _imm2, _:6,5;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                rs2,_:4, 2;
                rs1,_:9, 7;
             }
             fn imm(&self)->InsnT {
                self._imm1() << 2 | self._imm2()
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, CB) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _imm1, _:12, 10;
                _imm2, _:6,1;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                rs1,_:9, 7;
             }
             fn imm(&self)->InsnT {
                self._imm1() << 6 | self._imm2()
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, CJ) => {
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                imm,_:12, 2;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
        }
    };
    ($name:ident, $($t:tt)*) => {
        Invalid_Format_Type!
    };
}