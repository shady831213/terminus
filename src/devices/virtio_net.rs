use terminus_spaceport::memory::region::Region;
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;
use terminus_spaceport::virtio::{Device, Queue, QueueClient, QueueSetting, Result, Error, DeviceAccess, MMIODevice, DescMeta};
use terminus_spaceport::devices::{TunTap, TUNTAP_MODE};
use std::io::ErrorKind;
use terminus_spaceport::irq::IrqVecSender;
use std::cell::RefCell;

#[derive(Default, Debug)]
#[repr(C)]
struct VirtIONetHeader {
    flags: u8,
    gso_type: u8,
    hdr_len: u16,
    gso_size: u16,
    csum_start: u16,
    csum_offset: u16,
    num_buffers: u16,
}

struct VirtIONetInputQueue {}

impl VirtIONetInputQueue {
    fn new() -> VirtIONetInputQueue {
        VirtIONetInputQueue {}
    }
}

impl QueueClient for VirtIONetInputQueue {
    fn receive(&self, _: &Queue, _: u16) -> Result<bool> {
        Ok(false)
    }
}

struct VirtIONetOutputQueue {
    tap: Rc<TunTap>,
    irq_sender: IrqVecSender,

}

impl VirtIONetOutputQueue {
    fn new(tap: &Rc<TunTap>, irq_sender: IrqVecSender) -> VirtIONetOutputQueue {
        VirtIONetOutputQueue {
            tap: tap.clone(),
            irq_sender,
        }
    }
}

impl QueueClient for VirtIONetOutputQueue {
    fn receive(&self, queue: &Queue, desc_head: u16) -> Result<bool> {
        let mut read_descs: Vec<DescMeta> = vec![];
        let mut write_descs: Vec<DescMeta> = vec![];
        let mut write_buffer:Vec<u8> = vec![];
        let mut read_buffer:Vec<u8> = vec![];
        let (_,write_len) = queue.extract(desc_head, &mut read_buffer, &mut write_buffer, &mut read_descs, &mut write_descs,false, true)?;
        let mut header = VirtIONetHeader::default();
        let header_size = std::mem::size_of::<VirtIONetHeader>();
        if write_len as usize >= header_size {
            unsafe { std::slice::from_raw_parts_mut((&mut header as *mut VirtIONetHeader) as *mut u8, header_size).copy_from_slice(&write_buffer[..header_size]) }
        } else {
            return Err(Error::ClientError("invalid net header!".to_string()));
        }
        self.tap.send(&write_buffer[header_size..]).unwrap();
        queue.set_used(desc_head, write_len as u32)?;
        queue.update_last_avail();
        self.irq_sender.send().unwrap();
        Ok(true)
    }
}

pub struct VirtIONetDevice {
    virtio_device: Device,
    tap: Rc<TunTap>,
    mac: RefCell<u64>,
    status: RefCell<u16>,
}

impl VirtIONetDevice {
    pub fn new(memory: &Rc<Region>, irq_sender: IrqVecSender, tap_name: &str, mac: u64) -> VirtIONetDevice {
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            1,
                                            1, 0, 1 << 5,
        );
        virtio_device.get_irq_vec().set_enable_uncheck(0, true);
        let input_queue = {
            let input = VirtIONetInputQueue::new();
            Queue::new(&memory, QueueSetting { max_queue_size: 1 }, input)
        };
        let tap = Rc::new(TunTap::new(tap_name, TUNTAP_MODE::Tap, false, true).unwrap());
        //must be larger than 2 + MAX_SKB_FRAGS, according to linux /drivers/net/virtio_net.c
        let output_queue = {
            let output = VirtIONetOutputQueue::new(&tap, virtio_device.get_irq_vec().sender(0).unwrap());
            Queue::new(&memory, QueueSetting { max_queue_size: 32 }, output)
        };
        virtio_device.add_queue(input_queue);
        virtio_device.add_queue(output_queue);
        VirtIONetDevice {
            virtio_device,
            tap,
            mac: RefCell::new(mac),
            status: RefCell::new(0),
        }
    }
    pub fn net_read(&self) {
        let input_queue = self.virtio_device.get_queue(0);
        if !input_queue.get_ready() {
            return;
        }
        let iter = input_queue.avail_iter().unwrap();
        for desc_head in iter {
            let mut read_descs: Vec<DescMeta> = vec![];
            let mut write_descs: Vec<DescMeta> = vec![];
            let mut write_buffer:Vec<u8> = vec![];
            let mut read_buffer:Vec<u8> = vec![];
            let (_,_) = input_queue.extract(desc_head, &mut read_buffer, &mut write_buffer, &mut read_descs, &mut write_descs,true, false).unwrap();
            let header_size = std::mem::size_of::<VirtIONetHeader>();
            //
            let ret = match self.tap.recv(&mut read_buffer[header_size..]) {
                Ok(size) => {
                    read_buffer.resize(size + header_size, 0);
                    size
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => 0,
                Err(e) => panic!("{:?}", e),
            };
            if ret > 0 {
                input_queue.copy_to(&read_descs, &read_buffer).unwrap();
                input_queue.set_used(desc_head, read_buffer.len() as u32).unwrap();
                input_queue.update_last_avail();
                self.virtio_device.get_irq_vec().sender(0).unwrap().send().unwrap();
            }
        }
    }
}

#[derive_io(Bytes, U32, U8)]
pub struct VirtIONet(Rc<VirtIONetDevice>);

impl VirtIONet {
    pub fn new(d: &Rc<VirtIONetDevice>) -> VirtIONet {
        VirtIONet(d.clone())
    }
}

impl DeviceAccess for VirtIONet {
    fn device(&self) -> &Device {
        &self.0.virtio_device
    }
    fn config(&self, offset: u64) -> u32 {
        let data = if offset < 6 {
            ((*self.0.mac.borrow() >> (offset << 3)) & self.config_mask(&offset)) as u32
        } else if offset >= 6 && offset < 8 {
            (((*self.0.status.borrow() as u64) >> (offset << 3)) & self.config_mask(&offset)) as u32
        } else {
            0
        };
        data
    }

    fn set_config(&self, offset: u64, val: &u32) {
        if offset < 6 {
            let mask = self.config_mask(&offset);
            *self.0.mac.borrow_mut() = *self.0.mac.borrow() & !mask | (((*val as u64) & mask) << offset)
        } else if offset >= 6 && offset < 8 {
            let mask = self.config_mask(&offset) as u16;
            *self.0.status.borrow_mut() = *self.0.status.borrow() & !mask | (((*val as u16) & mask) << (offset as u16))
        }
    }
}

impl MMIODevice for VirtIONet {}

impl BytesAccess for VirtIONet {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        self.write_bytes(addr, data);
        Ok(0)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        self.read_bytes(addr, data);
        Ok(0)
    }
}

impl U8Access for VirtIONet {
    fn write(&self, addr: &u64, data: u8) {
        MMIODevice::write(self, addr, &(data as u32))
    }

    fn read(&self, addr: &u64) -> u8 {
        MMIODevice::read(self, addr) as u8
    }
}

impl U32Access for VirtIONet {
    fn write(&self, addr: &u64, data: u32) {
        MMIODevice::write(self, addr, &data)
    }

    fn read(&self, addr: &u64) -> u32 {
        MMIODevice::read(self, addr)
    }
}
