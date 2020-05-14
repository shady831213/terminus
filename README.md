# terminus
A verification-friendly riscv isa simulator in rust.


## Boot Linux in 30s
![Boot Linux in 30s](video/linux_boot.gif)

## Getting Start

```
  git clone https://github.com/shady831213/terminus
  cd terminus
  cargo update -p terminus-spaceport
  cargo install --path .
  terminus examples/linux/image/br-5-4
  //or
  cd examples/linux/image
  tar -zxvf rootfs.ext4.gz
  cd -
  terminus examples/linux/image/br-5-4.disk --image=examples/linux/image/rootfs.ext4
```

### Multi-cores support
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
### net support
after instll terminus
config your host according to help message.
then
```
  terminus (TERMINUS_PATH)examples/linux/image/br-net --image=(TERMINUS_PATH)examples/linux/image/rootfs.ext4 --net=tap0 --boot_args="root=/dev/vda console=hvc0"
  //booting...
  //booting...
  //booting...
  //...
  buildroot login: root
  //password is terminus
  Password:
  //config guset according to help message.
  # ping -c 1 www.baidu.com
```
then you will see:
```
PING www.baidu.com (103.235.47.103): 56 data bytes
64 bytes from 103.235.47.103: seq=0 ttl=46 time=17.685 ms

--- www.baidu.com ping statistics ---
1 packets transmitted, 1 packets received, 0% packet loss
round-trip min/avg/max = 17.685/17.685/17.685 ms
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
- [x] VirtIO disk
- [x] VirtIO network
- [ ] framebuffer
- [ ] co-sim with RTL simulator
- [ ] debug mode
- [ ] other extensions(b, v ...)

## Notice
Kernel image 'examples/linux/image/br-net' is coming from [diskimage-linux-riscv-2018-09-23.tar.gz](https://bellard.org/tinyemu/diskimage-linux-riscv-2018-09-23.tar.gz) of [tinyemu](https://bellard.org/tinyemu/)

