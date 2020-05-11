# terminus
A verification-friendly riscv isa simulator in rust.


## Boot Linux in 40s
![Boot Linux in 40s](video/linux_boot.gif)

## Getting Start

```
  git clone https://github.com/shady831213/terminus
  cd terminus
  cargo update -p terminus-spaceport
  cargo install --path .
  terminus examples/linux/image/br-5-4
```

## Multi-cores support
```
  terminus examples/linux/image/br-5-4 -p 4
  //booting...
  //booting...
  //booting...
  //...
  buildroot login: root
  //password is terminus
  Password: 
  # cat /proc/cpuinfo
```
then you will see:
```
processor	: 0
hart		: 0
mmu		: sv48

processor	: 1
hart		: 1
mmu		: sv48

processor	: 2
hart		: 2
mmu		: sv48

processor	: 3
hart		: 3
mmu		: sv48

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
- [x] Boot Linux(smp)
- [ ] DPI support
- [ ] Publish to crate.io
- [x] PLIC
- [x] VirtIO console
- [ ] VirtIO disk
- [ ] VirtIO network
- [ ] VirtIO framebuffer
- [ ] co-sim with RTL simulator
- [ ] debug mode
- [ ] other extensions(b, v ...)




