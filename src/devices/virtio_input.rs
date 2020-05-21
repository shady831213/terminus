use terminus_spaceport::memory::region::Region;
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;
use terminus_spaceport::virtio::{Device, Queue, QueueClient, QueueSetting, Result, Error, DeviceAccess, MMIODevice, DescMeta};
use terminus_spaceport::irq::IrqVecSender;
use terminus_spaceport::devices::KeyBoard;

const VIRTIO_INPUT_EV_SYN: u16 = 0x00;
const VIRTIO_INPUT_EV_KEY: u16 = 0x01;
const VIRTIO_INPUT_EV_REL: u16 = 0x02;
const VIRTIO_INPUT_EV_ABS: u16 = 0x03;
const VIRTIO_INPUT_EV_REP: u16 = 0x14;

struct VirtIOInputInputQueue {}

impl VirtIOInputInputQueue {
    fn new() -> VirtIOInputInputQueue {
        VirtIOInputInputQueue {}
    }
}

impl QueueClient for VirtIOInputInputQueue {
    fn receive(&self, _: &Queue, _: u16) -> Result<bool> {
        Ok(false)
    }
}

struct VirtIOInputOutputQueue {
    irq_sender: IrqVecSender,

}

impl VirtIOInputOutputQueue {
    fn new(irq_sender: IrqVecSender) -> VirtIOInputOutputQueue {
        VirtIOInputOutputQueue {
            irq_sender,
        }
    }
}

impl QueueClient for VirtIOInputOutputQueue {
    fn receive(&self, queue: &Queue, desc_head: u16) -> Result<bool> {
        queue.set_used(desc_head, 0)?;
        queue.update_last_avail();
        self.irq_sender.send().unwrap();
        Ok(true)
    }
}

trait VirtIOInputDevice {
    fn send_queue_envent(&self, device: &Device, ty: u16, code: u16, val: u32) -> bool {
        let input_queue = device.get_queue(0);
        if !input_queue.get_ready() {
            return false;
        }
        if let Some(desc_head) = input_queue.avail_iter().unwrap().next() {
            let mut read_descs: Vec<DescMeta> = vec![];
            let mut write_descs: Vec<DescMeta> = vec![];
            let mut write_buffer: Vec<u8> = vec![];
            let mut read_buffer: Vec<u8> = vec![];
            let (read_len, _) = input_queue.extract(desc_head, &mut read_buffer, &mut write_buffer, &mut read_descs, &mut write_descs, true, false).unwrap();
            assert!(read_len >= 8);
            read_buffer[0..2].copy_from_slice(&ty.to_le_bytes());
            read_buffer[2..4].copy_from_slice(&code.to_le_bytes());
            read_buffer[4..8].copy_from_slice(&val.to_le_bytes());
            read_buffer.resize(8, 0);
            input_queue.copy_to(&read_descs, &read_buffer).unwrap();
            input_queue.set_used(desc_head, read_buffer.len() as u32).unwrap();
            input_queue.update_last_avail();
            device.get_irq_vec().sender(0).unwrap().send().unwrap();
            true
        } else {
            false
        }
    }
}

pub struct VirtIOKbDevice {
    virtio_device: Device,
}

impl VirtIOKbDevice {
    pub fn new(memory: &Rc<Region>, irq_sender: IrqVecSender, tap_name: &str, mac: u64) -> VirtIOKbDevice {
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            1,
                                            1, 0, 1 << 5,
        );
        virtio_device.get_irq_vec().set_enable_uncheck(0, true);
        let input_queue = {
            let input = VirtIOInputInputQueue::new();
            Queue::new(&memory, QueueSetting { max_queue_size: 32 }, input)
        };
        let output_queue = {
            let output = VirtIOInputOutputQueue::new(virtio_device.get_irq_vec().sender(0).unwrap());
            Queue::new(&memory, QueueSetting { max_queue_size: 1 }, output)
        };
        virtio_device.add_queue(input_queue);
        virtio_device.add_queue(output_queue);
        VirtIOKbDevice {
            virtio_device,
        }
    }
}

impl VirtIOInputDevice for VirtIOKbDevice {}

impl KeyBoard for VirtIOKbDevice {
    fn send_key_event(&self, key_down: bool, val: u16) {
        // if !self.send_queue_envent(&self.virtio_device, VIRTIO_INPUT_EV_KEY, )
    }
}