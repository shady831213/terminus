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
             fn imm_len(&self)-> usize {
                12
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
            fn imm_len(&self)-> usize {
                12
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
            fn imm_len(&self)-> usize {
                13
            }
            fn ir(&self)->InsnT {
               self._ir()
            }
        }
    };
    ($name:ident, U) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _imm, _:31,12;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:6,0;
                rd,_:11, 7;
             }
             fn imm(&self)->InsnT {
                self._imm() << 12
             }
             fn imm_len(&self)-> usize {
                32
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
            fn imm_len(&self)-> usize {
                21
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
        impl $name {
             bitfield_fields!{
                InsnT;
                _rd,_:4, 2;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
                imm,_:12, 5;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
             fn rd(&self)->InsnT {
                self._rd() + 8
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
             fn imm_len(&self)-> usize {
                6
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
             fn imm_len(&self)-> usize {
                6
             }
        }
    };
    ($name:ident, CL) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _rd,_:4, 2;
                _rs1,_:9, 7;
                _imm1, _:12, 10;
                _imm2, _:6,5;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
             }
             fn imm(&self)->InsnT {
                self._imm1() << 2 | self._imm2()
             }
             fn imm_len(&self)-> usize {
                5
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
             fn rd(&self)->InsnT {
                self._rd() + 8
             }
             fn rs1(&self)->InsnT {
                self._rs1() + 8
             }
        }
    };
    ($name:ident, CS) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _rs2,_:4, 2;
                _rs1,_:9, 7;
                _imm1, _:12, 10;
                _imm2, _:6,5;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
             }
             fn imm(&self)->InsnT {
                self._imm1() << 2 | self._imm2()
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
             fn rs2(&self)->InsnT {
                self._rs2() + 8
             }
             fn rs1(&self)->InsnT {
                self._rs1() + 8
             }
             fn imm_len(&self)-> usize {
                5
             }
        }
    };
    ($name:ident, CB) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _rs1,_:9, 7;
                _imm1, _:12, 10;
                _imm2, _:6,1;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
             }
             fn imm(&self)->InsnT {
                self._imm1() << 6 | self._imm2()
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
             fn rs1(&self)->InsnT {
                self._rs1() + 8
             }
        }
    };
    ($name:ident, CA) => {
        impl $name {
             bitfield_fields!{
                InsnT;
                _rs2,_:4, 2;
                _rs1,_:9, 7;
                _rd,_:9, 7;
             }
        }
        impl Format for $name {
             bitfield_fields!{
                InsnT;
                op,_:1,0;
             }
             fn ir(&self)->InsnT {
                self._ir()
             }
             fn rs1(&self)->InsnT {
                self._rs1() + 8
             }
             fn rs2(&self)->InsnT {
                self._rs2() + 8
             }
             fn rd(&self)->InsnT {
                self._rd() + 8
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
             fn imm_len(&self)-> usize {
                11
             }
        }
    };
    ($name:ident, $($t:tt)*) => {
        Invalid_Format_Type!
    };
}