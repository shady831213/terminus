use terminus_spaceport::memory::region::Region;
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;
use terminus_spaceport::virtio::{Device, Queue, QueueClient, QueueSetting, Result, DeviceAccess, MMIODevice};
use terminus_spaceport::devices::TERM;
use std::io::{Write, ErrorKind, Read};
use std::ops::Deref;
use std::cmp::min;
use terminus_spaceport::irq::IrqVecSender;
use std::cell::RefCell;


struct VirtIOInputQueue {}

impl VirtIOInputQueue {
    fn new() -> VirtIOInputQueue {
        VirtIOInputQueue {}
    }
}

impl QueueClient for VirtIOInputQueue {
    fn receive(&self, _: &Queue, _: u16) -> Result<bool> {
        //eprintln!("virtio console input!");
        Ok(false)
    }
}

struct VirtIOOutputQueue {
    memory: Rc<Region>,
}

impl VirtIOOutputQueue {
    fn new(memory: &Rc<Region>) -> VirtIOOutputQueue {
        VirtIOOutputQueue {
            memory: memory.clone(),
        }
    }
}

impl QueueClient for VirtIOOutputQueue {
    fn receive(&self, queue: &Queue, desc_head: u16) -> Result<bool> {
        //eprintln!("virtio console output!");
        let desc = queue.get_desc(desc_head)?;
        if desc.len > 0 {
            let mut buffer: Vec<u8> = vec![0; desc.len as usize];
            BytesAccess::read(self.memory.deref(), &desc.addr, &mut buffer);
            let stdout = TERM.stdout();
            let mut handle = stdout.lock();
            handle.write(&buffer).unwrap();
            handle.flush().unwrap();
        }
        queue.set_used(desc_head, desc.len)?;
        queue.update_last_avail();
        Ok(true)
    }
}

pub struct VirtIOConsoleDevice {
    virtio_device: Device,
    width: u32,
    hight: u32,
    resize_pending: RefCell<bool>,
    resize_irq: IrqVecSender,
}

impl VirtIOConsoleDevice {
    pub fn new(memory: &Rc<Region>, irq_sender: IrqVecSender) -> VirtIOConsoleDevice {
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            2,
                                            3, 0, 1,
        );
        let input_queue = {
            let input = VirtIOInputQueue::new();
            Queue::new(&memory, QueueSetting { max_queue_size: 1 }, input)
        };
        let output_queue = {
            let output = VirtIOOutputQueue::new(memory);
            Queue::new(&memory, QueueSetting { max_queue_size: 1 }, output)
        };
        virtio_device.get_irq_vec().set_enable_uncheck(1, true);
        let resize_irq = virtio_device.get_irq_vec().sender(1).unwrap();
        virtio_device.add_queue(input_queue);
        virtio_device.add_queue(output_queue);
        VirtIOConsoleDevice {
            virtio_device,
            width: 1024,
            hight: 768,
            resize_pending: RefCell::new(true),
            resize_irq,
        }
    }
    pub fn console_read(&self) {
        let input_queue = self.virtio_device.get_queue(0);
        if !input_queue.get_ready() {
            return;
        }
        if let Some(desc_head) = input_queue.avail_iter().unwrap().last() {
            let desc = input_queue.get_desc(desc_head).unwrap();
            let len = min(desc.len as usize, 128);
            //eprintln!("virtio can read {}", len);
            let mut buffer: Vec<u8> = vec![0; len];
            let ret = match TERM.stdin().lock().read(&mut buffer) {
                Ok(l) => l,
                Err(e) if e.kind() == ErrorKind::WouldBlock => 0,
                Err(e) => panic!("{:?}", e)
            };
            if ret > 0 {
                BytesAccess::write(self.virtio_device.memory().deref(), &desc.addr, &buffer[..ret]);
                input_queue.set_used(desc_head, ret as u32).unwrap();
                input_queue.update_last_avail();
            }
        }
    }
    pub fn console_resize(&self) {
        let mut resize_pending = self.resize_pending.borrow_mut();
        if *resize_pending {
            let input_queue = self.virtio_device.get_queue(0);
            if !input_queue.get_ready() {
                return;
            }
            if input_queue.avail_iter().unwrap().last().is_some() {
                self.resize_irq.send().unwrap();
                *resize_pending = false;
            }
        }
    }
}

#[derive_io(Bytes, U32)]
pub struct VirtIOConsole(Rc<VirtIOConsoleDevice>);

impl VirtIOConsole {
    pub fn new(d: &Rc<VirtIOConsoleDevice>) -> VirtIOConsole {
        VirtIOConsole(d.clone())
    }
}

impl DeviceAccess for VirtIOConsole {
    fn device(&self) -> &Device {
        &self.0.virtio_device
    }
    fn config(&self, _: u64) -> u32 {
        self.0.hight << 16 | self.0.width
    }
}

impl MMIODevice for VirtIOConsole {}

impl BytesAccess for VirtIOConsole {
    fn write(&self, addr: &u64, data: &[u8]) {
        //eprintln!("virtio console write @ {:#x} {:#x?}!", *addr, data);
        if *addr == 0x50 {
            //eprintln!("virtio notify queue {}!", self.queue_sel());
        }
        self.write_bytes(addr, data)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) {
        self.read_bytes(addr, data);
        //eprintln!("virtio console read @ {:#x} {:#x?}!", *addr, data);
    }
}

impl U32Access for VirtIOConsole {
    fn write(&self, addr: &u64, data: u32) {
        assert!((*addr).trailing_zeros() > 1, format!("U32Access:unaligned addr:{:#x}", addr));
        self.write_u32(addr, &data)
    }

    fn read(&self, addr: &u64) -> u32 {
        assert!((*addr).trailing_zeros() > 1, format!("U32Access:unaligned addr:{:#x}", addr));
        self.read_u32(addr)
    }
}



