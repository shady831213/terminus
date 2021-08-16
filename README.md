# terminus
A RISC-V instruction set simulator in Rust.


## Boot Linux in 30s
![Boot Linux in 30s](video/linux_boot.gif)

## Getting Started

```
  git clone https://github.com/shady831213/terminus
  cd terminus
  cargo update -p terminus-spaceport
  cargo update -p terminus-vault
  cargo install --path .
  terminus examples/linux/image/br-5-4
  //or
  cd examples/linux/image
  tar -zxvf rootfs.ext4.gz
  cd -
  terminus examples/linux/image/br-5-4.disk --image=examples/linux/image/rootfs.ext4
```

### Multi-core Support

Specify the `-p` option to configure the number of RISC-V HARTs (HARdware Threads):

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
### Networking Support

Run the `setup_tuntap.sh` script to create a simulated network interface and networking bridge.

```
  terminus examples/linux/image/br-5-4.disk --image=examples/linux/image/rootfs.ext4 --net=tap0
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

### Display Support

Terminus with display supported needs the "sdl" operating system packages and related dependencies.

```
  // Example for  Ubuntu
  sudo apt-get install libsdl2-dev
  git clone https://github.com/shady831213/terminus
  cd terminus
  cargo update -p terminus-spaceport
  cargo install --features="sdl" --path .

  cd examples/linux/image
  tar -zxvf rootfs.ext4.gz
  cd -
  terminus examples/linux/image/br-5-4.disk --image=examples/linux/image/rootfs.ext4 --boot_args="root=/dev/vda console=tty0 earlycon=sbi" --display
```

### Cosimulation with HDL
Please refer to [terminus_cosim](https://github.com/shady831213/terminus_cosim/tree/master/terminus_cluster).

## RoadMap
- [x] RV32/64I
- [x] MADFC
- [x] M/S/U privilege
- [x] Pass all riscv_tests
- [x] CLINT and Timer
- [x] HTIF console
- [x] FDT generation
- [x] Multi Cores
- [x] Boot Linux
- [x] Emu mode binary
- [x] Boot Linux(smp)
- [ ] Publish to crate.io
- [x] PLIC
- [x] VirtIO console
- [x] VirtIO disk
- [x] VirtIO network
- [x] framebuffer
- [x] VirtIO keyboard
- [x] VirtIO mouse
- [x] Cosimulation with HDL
- [ ] debug mode
- [ ] other extensions(b, v ...)


