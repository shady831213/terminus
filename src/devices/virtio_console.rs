use terminus_spaceport::memory::region::Region;
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;
use terminus_spaceport::virtio::{Device, Queue, QueueClient, QueueSetting, Result, DeviceAccess, MMIODevice};
use terminus_spaceport::devices::TERM;
use std::io::{Write, ErrorKind, Read};
use std::ops::Deref;
use std::cmp::min;


struct VirtIOInputQueue {}

impl VirtIOInputQueue {
    fn new() -> VirtIOInputQueue {
        VirtIOInputQueue {}
    }
}

impl QueueClient for VirtIOInputQueue {
    fn receive(&self, _: &Queue, _: u16) -> Result<bool> {
        Ok(true)
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
        Ok(true)
    }
}

struct VirtIOConsoleDevice {
    virtio_device: Device,
}

impl VirtIOConsoleDevice {
    pub fn new(memory: &Rc<Region>) -> VirtIOConsoleDevice {
        let mut virtio_device = Device::new(memory,
                                            None,
                                            0,
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
        virtio_device.add_queue(input_queue);
        virtio_device.add_queue(output_queue);

        virtio_device.get_irq_vec().set_enable(1, true).unwrap();
        VirtIOConsoleDevice {
            virtio_device,
        }
    }
    pub fn console_read(&self) {
        let input_queue = self.virtio_device.get_queue(0);
        if let Some(desc_head) = input_queue.avail_iter().unwrap().last() {
            let desc = input_queue.get_desc(desc_head).unwrap();
            let len = min(desc.len as usize, 128);
            let mut buffer: Vec<u8> = vec![0; len];
            let ret = match TERM.stdin().lock().read(&mut buffer) {
                Ok(l) => l,
                Err(e) if e.kind() == ErrorKind::WouldBlock => 0,
                Err(e) => panic!("{:?}", e)
            };
            if ret > 0 {
                BytesAccess::write(self.virtio_device.memory().deref(), &desc.addr, &buffer[..ret]);
                input_queue.set_used(desc_head, ret as u32).unwrap()
            }
        }
    }
}

#[derive_io(Bytes, U32)]
struct VirtIOConsole(Rc<VirtIOConsoleDevice>);

impl VirtIOConsole {
    pub fn new(d: &Rc<VirtIOConsoleDevice>) -> VirtIOConsole {
        VirtIOConsole(d.clone())
    }
}

impl DeviceAccess for VirtIOConsole {
    fn device(&self) -> &Device {
        &self.0.virtio_device
    }
}

impl MMIODevice for VirtIOConsole {}

impl BytesAccess for VirtIOConsole {
    fn write(&self, addr: &u64, data: &[u8]) {
        self.write_bytes(addr, data)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) {
        self.read_bytes(addr, data)
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



