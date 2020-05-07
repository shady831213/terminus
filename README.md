# terminus
A verification-friendly riscv isa simulator in rust.


## Boot Linux in 40s
![Boot Linux in 40s](video/linux_boot.gif)

## Getting Start

```
  git clone https://github.com/shady831213/terminus
  cd terminus
  cargo install --path .
  terminus examples/linux/image/br-5-4
```

## RoadMap
- [x] RV32/64I
- [x] MADFC
- [x] Pass all riscv_tests
- [x] CLINT and Timer
- [x] HTIF console
- [x] FDT generation
- [x] Multi Cores
- [x] Boot Linux
- [x] Emu mode binary
- [ ] Boot Linux(smp)
- [ ] DPI support
- [ ] Publish to crate.io
- [ ] PLIC
- [ ] VirtIO console
- [ ] VirtIO disk
- [ ] VirtIO network
- [ ] VirtIO framebuffer
- [ ] co-sim with RTL simulator
- [ ] debug mode
- [ ] other extensions(b, v ...)




