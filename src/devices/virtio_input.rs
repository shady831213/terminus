use terminus_spaceport::memory::region::Region;
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;
use terminus_spaceport::virtio::{Device, Queue, QueueClient, QueueSetting, Result, DeviceAccess, MMIODevice, DescMeta};
use terminus_spaceport::irq::IrqVecSender;
use terminus_spaceport::devices::{KeyBoard, MAX_ABS_SCALE, Mouse};
use std::cell::RefCell;
use std::ops::DerefMut;

const VIRTIO_INPUT_EV_SYN: u8 = 0x00;
const VIRTIO_INPUT_EV_KEY: u8 = 0x01;
const VIRTIO_INPUT_EV_REL: u8 = 0x02;
const VIRTIO_INPUT_EV_ABS: u8 = 0x03;
const VIRTIO_INPUT_EV_REP: u8 = 0x14;

const VIRTIO_INPUT_CFG_UNSET: u8 = 0x00;
const VIRTIO_INPUT_CFG_ID_NAME: u8 = 0x01;
const VIRTIO_INPUT_CFG_ID_SERIAL: u8 = 0x02;
const VIRTIO_INPUT_CFG_ID_DEVIDS: u8 = 0x03;
const VIRTIO_INPUT_CFG_PROP_BITS: u8 = 0x10;
const VIRTIO_INPUT_CFG_EV_BITS: u8 = 0x11;
const VIRTIO_INPUT_CFG_ABS_INFO: u8 = 0x12;

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

    fn config_id_name(&self) -> &str;

    fn config_ev_write(&self, config: &mut [u8]);

    fn config_abs_write(&self, _: &mut [u8]) {}

    fn config_write(&self, config: &mut [u8]) {
        match config[0] {
            VIRTIO_INPUT_CFG_UNSET => {}
            VIRTIO_INPUT_CFG_ID_NAME => {
                let name = self.config_id_name();
                let len = name.len();
                config[2] = len as u8;
                config[8..8 + len].copy_from_slice(name.as_bytes())
            }
            VIRTIO_INPUT_CFG_ID_SERIAL | VIRTIO_INPUT_CFG_ID_DEVIDS | VIRTIO_INPUT_CFG_PROP_BITS => {
                config[2] = 0
            }
            VIRTIO_INPUT_CFG_EV_BITS => {
                config[2] = 0;
                self.config_ev_write(config)
            }
            VIRTIO_INPUT_CFG_ABS_INFO => {
                self.config_abs_write(config)
            }
            _ => {}
        }
    }
}

pub struct VirtIOKbDevice {
    virtio_device: Device,
    config: RefCell<[u8; 256]>,
}

impl VirtIOKbDevice {
    pub fn new(memory: &Rc<Region>, irq_sender: IrqVecSender) -> VirtIOKbDevice {
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            1,
                                            18, 0, 0,
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
            config: RefCell::new([0; 256]),
        }
    }
}

impl VirtIOInputDevice for VirtIOKbDevice {
    fn config_id_name(&self) -> &str {
        "virtio_keyboard"
    }

    fn config_ev_write(&self, config: &mut [u8]) {
        match config[1] {
            VIRTIO_INPUT_EV_KEY => {
                config[2] = (256 >> 3) as u8;
                for i in 8..8 + (256 >> 3) {
                    config[i] = 0xff
                }
            }
            VIRTIO_INPUT_EV_REP => {
                config[2] = 1;
            }
            _ => {}
        }
    }
}

impl KeyBoard for VirtIOKbDevice {
    fn send_key_event(&self, key_down: bool, val: u16) {
        if self.send_queue_envent(&self.virtio_device, VIRTIO_INPUT_EV_KEY as u16, val, key_down as u32) {
            self.send_queue_envent(&self.virtio_device, VIRTIO_INPUT_EV_SYN as u16, 0, 0);
        }
    }
}

#[derive_io(Bytes)]
pub struct VirtIOKb(Rc<VirtIOKbDevice>);

impl VirtIOKb {
    pub fn new(d: &Rc<VirtIOKbDevice>) -> VirtIOKb {
        VirtIOKb(d.clone())
    }
}

impl DeviceAccess for VirtIOKb {
    fn device(&self) -> &Device {
        &self.0.virtio_device
    }
    fn config(&self, offset: u64, data: &mut [u8]) {
        let len = data.len();
        let off = offset as usize;
        if off < 256 && off + len <= 256 {
            data.copy_from_slice(&(*self.0.config.borrow())[off..off + len])
        }
    }

    fn set_config(&self, offset: u64, data: &[u8]) {
        let len = data.len();
        let off = offset as usize;
        let mut config = self.0.config.borrow_mut();
        if off < 256 && off + len <= 256 {
            (*config)[off..off + len].copy_from_slice(data);
            self.0.config_write(config.deref_mut())
        }
    }
}

impl MMIODevice for VirtIOKb {}

impl BytesAccess for VirtIOKb {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        self.write_bytes(addr, data);
        Ok(0)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        self.read_bytes(addr, data);
        Ok(0)
    }
}

const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;
const BTN_GEAR_DOWN: u16 = 0x150;
const BTN_GEAR_UP: u16 = 0x151;

// const REL_X: u8 = 0x00;
// const REL_Y: u8 = 0x01;
// const REL_Z: u8 = 0x02;
const REL_WHEEL: u8 = 0x08;

const ABS_X: u8 = 0x00;
const ABS_Y: u8 = 0x01;
// const ABS_Z: u8 = 0x02;


pub struct VirtIOMouseDevice {
    virtio_device: Device,
    buttons: RefCell<u32>,
    valid_buttons: Vec<u16>,
    config: RefCell<[u8; 256]>,
}

impl VirtIOMouseDevice {
    pub fn new(memory: &Rc<Region>, irq_sender: IrqVecSender) -> VirtIOMouseDevice {
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            1,
                                            18, 0, 0,
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
        VirtIOMouseDevice {
            virtio_device,
            buttons: RefCell::new(0),
            valid_buttons: vec![BTN_LEFT, BTN_RIGHT, BTN_MIDDLE, BTN_GEAR_DOWN, BTN_GEAR_UP],
            config: RefCell::new([0; 256]),
        }
    }
}

impl VirtIOInputDevice for VirtIOMouseDevice {
    fn config_id_name(&self) -> &str {
        "virtio_mouse"
    }

    fn config_ev_write(&self, config: &mut [u8]) {
        match config[1] {
            VIRTIO_INPUT_EV_KEY => {
                config[2] = (512 >> 3) as u8;
                for i in 8..8 + (512 >> 3) {
                    config[i] = 0x0
                }
                for b in self.valid_buttons.iter() {
                    config[8 + (*b >> 3) as usize] |= (1 << (*b & 0x7)) as u8
                }
            }
            VIRTIO_INPUT_EV_REL => {
                config[2] = 2;
                config[8] = 0;
                config[9] = 0;
                // config[8 + (REL_X >> 3) as usize] |= (1 << (REL_X & 0x7)) as u8;
                // config[8 + (REL_Y >> 3) as usize] |= (1 << (REL_Y & 0x7)) as u8;
                config[8 + (REL_WHEEL >> 3) as usize] |= (1 << (REL_WHEEL & 0x7)) as u8;
            }
            VIRTIO_INPUT_EV_ABS => {
                config[2] = 1;
                config[8] = 0;
                config[8] |= (1 << (ABS_X & 0x7)) as u8;
                config[8] |= (1 << (ABS_Y & 0x7)) as u8;
            }
            _ => {}
        }
    }

    fn config_abs_write(&self, config: &mut [u8]) {
        if config[1] <= 1 {
            config[2] = 5 * 4;
            //min
            config[8..8 + 4].copy_from_slice(&(0 as u32).to_le_bytes());
            //max
            config[8 + 4..8 + 8].copy_from_slice(&(MAX_ABS_SCALE - 1).to_le_bytes());
            //fuzz
            config[8 + 8..8 + 12].copy_from_slice(&(0 as u32).to_le_bytes());
            //flat
            config[8 + 12..8 + 16].copy_from_slice(&(0 as u32).to_le_bytes());
            //res
            config[8 + 16..8 + 20].copy_from_slice(&(0 as u32).to_le_bytes());
        }
    }
}

impl Mouse for VirtIOMouseDevice {
    fn send_mouse_event(&self, x: i32, y: i32, z: i32, buttons: u32) {
        let mut mouse_buttons = buttons;

        if z != 0 {
            if z > 0 {
                mouse_buttons |= 1 << 3;
            } else {
                mouse_buttons |= 1 << 4;
            }
            let ret = self.send_queue_envent(&self.virtio_device, VIRTIO_INPUT_EV_REL as u16, REL_WHEEL as u16, z as u32);
            if !ret {
                return;
            }
        } else {
            let ret = self.send_queue_envent(&self.virtio_device, VIRTIO_INPUT_EV_ABS as u16, ABS_X as u16, x as u32);
            if !ret {
                return;
            }
            let ret = self.send_queue_envent(&self.virtio_device, VIRTIO_INPUT_EV_ABS as u16, ABS_Y as u16, y as u32);
            if !ret {
                return;
            }
        }

        let mut cur_buttons = self.buttons.borrow_mut();
        if mouse_buttons != *cur_buttons {
            //left, right, middle
            for (i, b) in self.valid_buttons.iter().enumerate() {
                let button = (mouse_buttons >> i as u32) & 1;
                let cur_button = (*cur_buttons >> i as u32) & 1;
                if button != cur_button {
                    let ret = self.send_queue_envent(&self.virtio_device, VIRTIO_INPUT_EV_KEY as u16, *b, button);
                    if !ret {
                        return;
                    }
                }
            }
            *cur_buttons = mouse_buttons
        }
        self.send_queue_envent(&self.virtio_device, VIRTIO_INPUT_EV_SYN as u16, 0, 0);
    }
    fn mouse_absolute(&self) -> bool { true }
}

#[derive_io(Bytes)]
pub struct VirtIOMouse(Rc<VirtIOMouseDevice>);

impl VirtIOMouse {
    pub fn new(d: &Rc<VirtIOMouseDevice>) -> VirtIOMouse {
        VirtIOMouse(d.clone())
    }
}

impl DeviceAccess for VirtIOMouse {
    fn device(&self) -> &Device {
        &self.0.virtio_device
    }
    fn config(&self, offset: u64, data: &mut [u8]) {
        let len = data.len();
        let off = offset as usize;
        if off < 256 && off + len <= 256 {
            data.copy_from_slice(&(*self.0.config.borrow())[off..off + len])
        }
    }

    fn set_config(&self, offset: u64, data: &[u8]) {
        let len = data.len();
        let off = offset as usize;
        let mut config = self.0.config.borrow_mut();
        if off < 256 && off + len <= 256 {
            (*config)[off..off + len].copy_from_slice(data);
            self.0.config_write(config.deref_mut())
        }
    }
}

impl MMIODevice for VirtIOMouse {}

impl BytesAccess for VirtIOMouse {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        self.write_bytes(addr, data);
        Ok(0)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        self.read_bytes(addr, data);
        Ok(0)
    }
}
