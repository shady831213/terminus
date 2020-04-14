use terminus_global::*;
use terminus_proc_macros::{define_csr, csr_map};
use terminus_macros::*;
csr_map! {
pub FCsrs(0x0, 0xfff) {
    fflags(RW):Fflags,0x001;
    frm(RW):Frm, 0x002;
    fcsr(RW):Fcsr,0x003;
}
}

define_csr! {
Fcsr {
    fields{
        nx(RW):0, 0;
        uf(RW):1, 1;
        of(RW):2, 2;
        dz(RW):3, 3;
        nv(RW):4, 4;
        frm(RW):7,5;
    }
}
}

define_csr! {
Fflags {
    fields{
        nx(RW):0, 0;
        uf(RW):1, 1;
        of(RW):2, 2;
        dz(RW):3, 3;
        nv(RW):4, 4;
    }
}
}

define_csr! {
Frm {
    fields{
        frm(RW):2,0;
    }
}
}