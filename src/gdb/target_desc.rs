pub const TARGET_DESCRIPTION_XML: &str = r#"<?xml version="1.0"?>
<!DOCTYPE target SYSTEM "gdb-target.dtd">
<target>
  <architecture>riscv:rv64</architecture>
  <feature name="org.gnu.gdb.riscv.cpu">
    <reg name="x0" bitsize="64" type="int" />
    <reg name="x1" bitsize="64" type="int" />
    <reg name="x2" bitsize="64" type="int" />
    <reg name="x3" bitsize="64" type="int" />
    <reg name="x4" bitsize="64" type="int" />
    <reg name="x5" bitsize="64" type="int" />
    <reg name="x6" bitsize="64" type="int" />
    <reg name="x7" bitsize="64" type="int" />
    <reg name="x8" bitsize="64" type="int" />
    <reg name="x9" bitsize="64" type="int" />
    <reg name="x10" bitsize="64" type="int" />
    <reg name="x11" bitsize="64" type="int" />
    <reg name="x12" bitsize="64" type="int" />
    <reg name="x13" bitsize="64" type="int" />
    <reg name="x14" bitsize="64" type="int" />
    <reg name="x15" bitsize="64" type="int" />
    <reg name="x16" bitsize="64" type="int" />
    <reg name="x17" bitsize="64" type="int" />
    <reg name="x18" bitsize="64" type="int" />
    <reg name="x19" bitsize="64" type="int" />
    <reg name="x20" bitsize="64" type="int" />
    <reg name="x21" bitsize="64" type="int" />
    <reg name="x22" bitsize="64" type="int" />
    <reg name="x23" bitsize="64" type="int" />
    <reg name="x24" bitsize="64" type="int" />
    <reg name="x25" bitsize="64" type="int" />
    <reg name="x26" bitsize="64" type="int" />
    <reg name="x27" bitsize="64" type="int" />
    <reg name="x28" bitsize="64" type="int" />
    <reg name="x29" bitsize="64" type="int" />
    <reg name="x30" bitsize="64" type="int" />
    <reg name="x31" bitsize="64" type="int" />
    <reg name="pc" bitsize="64" type="code_ptr" regnum="32"/>
  </feature>
  <feature name="org.gnu.gdb.riscv.fpu">
    <reg name="f0" bitsize="64" type="ieee_double" />
    <reg name="f1" bitsize="64" type="ieee_double" />
    <reg name="f2" bitsize="64" type="ieee_double" />
    <reg name="f3" bitsize="64" type="ieee_double" />
    <reg name="f4" bitsize="64" type="ieee_double" />
    <reg name="f5" bitsize="64" type="ieee_double" />
    <reg name="f6" bitsize="64" type="ieee_double" />
    <reg name="f7" bitsize="64" type="ieee_double" />
    <reg name="f8" bitsize="64" type="ieee_double" />
    <reg name="f9" bitsize="64" type="ieee_double" />
    <reg name="f10" bitsize="64" type="ieee_double" />
    <reg name="f11" bitsize="64" type="ieee_double" />
    <reg name="f12" bitsize="64" type="ieee_double" />
    <reg name="f13" bitsize="64" type="ieee_double" />
    <reg name="f14" bitsize="64" type="ieee_double" />
    <reg name="f15" bitsize="64" type="ieee_double" />
    <reg name="f16" bitsize="64" type="ieee_double" />
    <reg name="f17" bitsize="64" type="ieee_double" />
    <reg name="f18" bitsize="64" type="ieee_double" />
    <reg name="f19" bitsize="64" type="ieee_double" />
    <reg name="f20" bitsize="64" type="ieee_double" />
    <reg name="f21" bitsize="64" type="ieee_double" />
    <reg name="f22" bitsize="64" type="ieee_double" />
    <reg name="f23" bitsize="64" type="ieee_double" />
    <reg name="f24" bitsize="64" type="ieee_double" />
    <reg name="f25" bitsize="64" type="ieee_double" />
    <reg name="f26" bitsize="64" type="ieee_double" />
    <reg name="f27" bitsize="64" type="ieee_double" />
    <reg name="f28" bitsize="64" type="ieee_double" />
    <reg name="f29" bitsize="64" type="ieee_double" />
    <reg name="f30" bitsize="64" type="ieee_double" />
    <reg name="f31" bitsize="64" type="ieee_double" />
    <reg name="fflags" bitsize="32" type="int" regnum="65"/>
    <reg name="frm" bitsize="32" type="int" regnum="66"/>
    <reg name="fcsr" bitsize="32" type="int" regnum="67"/>
  </feature>
  <feature name="org.gnu.gdb.riscv.csr">
    <reg name="sstatus" bitsize="64" type="int" regnum="66"/>
    <reg name="sie" bitsize="64" type="int" regnum="67"/>
    <reg name="stvec" bitsize="64" type="int" regnum="68"/>
    <reg name="sscratch" bitsize="64" type="int" regnum="69"/>
    <reg name="sepc" bitsize="64" type="int" regnum="70"/>
    <reg name="scause" bitsize="64" type="int" regnum="71"/>
    <reg name="stval" bitsize="64" type="int" regnum="72"/>
    <reg name="sip" bitsize="64" type="int" regnum="73"/>
    <reg name="satp" bitsize="64" type="int" regnum="74"/>
    <reg name="mstatus" bitsize="64" type="int" regnum="75"/>
    <reg name="misa" bitsize="64" type="int" regnum="76"/>
    <reg name="medeleg" bitsize="64" type="int" regnum="77"/>
    <reg name="mideleg" bitsize="64" type="int" regnum="78"/>
    <reg name="mie" bitsize="64" type="int" regnum="79"/>
    <reg name="mtvec" bitsize="64" type="int" regnum="80"/>
    <reg name="mscratch" bitsize="64" type="int" regnum="81"/>
    <reg name="mepc" bitsize="64" type="int" regnum="82"/>
    <reg name="mcause" bitsize="64" type="int" regnum="83"/>
    <reg name="mtval" bitsize="64" type="int" regnum="84"/>
    <reg name="mip" bitsize="64" type="int" regnum="85"/>
    <reg name="mcycle" bitsize="64" type="int" regnum="86"/>
    <reg name="minstret" bitsize="64" type="int" regnum="87"/>
    <reg name="mvendorid" bitsize="64" type="int" regnum="88"/>
    <reg name="marchid" bitsize="64" type="int" regnum="89"/>
    <reg name="mimpid" bitsize="64" type="int" regnum="90"/>
    <reg name="mhartid" bitsize="64" type="int" regnum="91"/>
  </feature>
  <feature name="org.gnu.gdb.riscv.virtual">
    <reg name="priv" bitsize="8" type="int" regnum="132"/>
  </feature>
</target>"#;
