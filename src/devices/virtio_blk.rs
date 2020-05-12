use terminus_spaceport::memory::region::{Region, GHEAP};
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;
use terminus_spaceport::virtio::{QueueClient, Queue, Result, Error, DESC_F_WRITE, DescMeta, Device, QueueSetting, DeviceAccess, MMIODevice};
use std::ops::Deref;
use terminus_spaceport::irq::IrqVecSender;
use std::fs;

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;
//SECTOR_SIZE= 512
const VIRTIO_BLK_SECTOR_SHIFT: u32 = 9;

const VIRTIO_BLK_S_OK: u8 = 0;
const VIRTIO_BLK_S_IOERR: u8 = 1;

#[derive(Default, Debug)]
struct VirtIOBlkHeader {
    ty: u32,
    ioprio: u32,
    sector_num: u32,
}

struct VirtIOBlkDiskSnapshot {
    snapshot: Rc<Region>,
}

impl VirtIOBlkDiskSnapshot {
    fn new(snapshot: &Rc<Region>) -> VirtIOBlkDiskSnapshot {
        VirtIOBlkDiskSnapshot { snapshot: snapshot.clone() }
    }
}

impl BytesAccess for VirtIOBlkDiskSnapshot {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        if *addr + data.len() as u64 > self.snapshot.info.size {
            Err("out of range!".to_string())
        } else {
            BytesAccess::write(self.snapshot.deref(), addr, data)
        }
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        if *addr + data.len() as u64 > self.snapshot.info.size {
            Err("out of range!".to_string())
        } else {
            BytesAccess::read(self.snapshot.deref(), addr, data)
        }
    }
}

struct VirtIOBlkQueue<T:BytesAccess> {
    memory: Rc<Region>,
    disk: T,
    irq_sender: IrqVecSender,
}

impl<T:BytesAccess> VirtIOBlkQueue<T> {
    fn new(memory: &Rc<Region>, disk: T, irq_sender: IrqVecSender) -> VirtIOBlkQueue<T> {
        VirtIOBlkQueue {
            memory: memory.clone(),
            disk,
            irq_sender,
        }
    }
}

impl<T:BytesAccess> QueueClient for VirtIOBlkQueue<T> {
    fn receive(&self, queue: &Queue, desc_head: u16) -> Result<bool> {
        let mut read_descs: Vec<DescMeta> = vec![];
        let mut read_len: u32 = 0;
        let mut write_descs: Vec<DescMeta> = vec![];
        let mut write_len: u32 = 0;
        for desc_res in queue.desc_iter(desc_head) {
            let (_, desc) = desc_res?;
            if desc.flags & DESC_F_WRITE != 0 {
                read_len += desc.len;
                read_descs.push(desc);
            } else {
                write_len += desc.len;
                write_descs.push(desc);
            }
        }
        let mut read_buffer: Vec<u8> = vec![0; read_len as usize];
        let write_buffer: Vec<u8> = {
            let mut buffer: Vec<u8> = vec![0; write_len as usize];
            let mut offset: usize = 0;
            for desc in write_descs.iter() {
                let next_offset = offset + desc.len as usize;
                BytesAccess::read(self.memory.deref(), &desc.addr, &mut buffer[offset..next_offset]).unwrap();
                offset = next_offset;
            }
            buffer
        };

        let mut header = VirtIOBlkHeader::default();
        let header_size = std::mem::size_of::<VirtIOBlkHeader>();
        if write_len as usize > header_size {
            unsafe { std::slice::from_raw_parts_mut((&mut header as *mut VirtIOBlkHeader) as *mut u8, header_size).copy_from_slice(&write_buffer[..header_size]) }
        } else {
            return Err(Error::ClientError("invalid block header!".to_string()));
        }

        let disk_offset = (header.sector_num << VIRTIO_BLK_SECTOR_SHIFT) as u64;

        match header.ty {
            VIRTIO_BLK_T_IN => {
                if BytesAccess::read(&self.disk, &disk_offset, &mut read_buffer[..read_len as usize - 1]).is_ok() {
                    read_buffer[read_len as usize - 1] = VIRTIO_BLK_S_OK;
                } else {
                    read_buffer[read_len as usize - 1] = VIRTIO_BLK_S_IOERR;
                }
                let mut offset: usize = 0;
                for desc in read_descs.iter() {
                    let next_offset = offset + desc.len as usize;
                    BytesAccess::write(self.memory.deref(), &desc.addr, &read_buffer[offset..next_offset]).unwrap();
                    offset = next_offset;
                }
                queue.set_used(desc_head, read_len)?;
            }
            VIRTIO_BLK_T_OUT => {
                if BytesAccess::write(&self.disk, &disk_offset, &write_buffer[header_size..]).is_ok() {
                    U8Access::write(self.memory.deref(), &read_descs.first().unwrap().addr, VIRTIO_BLK_S_OK);
                } else {
                    U8Access::write(self.memory.deref(), &read_descs.first().unwrap().addr, VIRTIO_BLK_S_IOERR);
                }
                queue.set_used(desc_head, 1)?;
            }
            _ => return Err(Error::ClientError(format!("invalid block ty {:#x}!", header.ty)))
        }
        queue.update_last_avail();
        self.irq_sender.send().unwrap();
        Ok(true)
    }
}

#[derive_io(Bytes, U32)]
pub struct VirtIOBlk {
    virtio_device: Device,
}

impl VirtIOBlk {
    pub fn new(memory: &Rc<Region>, irq_sender: IrqVecSender, num_queues: usize, file_name: &str) -> VirtIOBlk {
        assert!(num_queues > 0);
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            1,
                                            2, 0, 8,
        );
        virtio_device.get_irq_vec().set_enable_uncheck(0, true);
        let content = fs::read(file_name).unwrap().into_boxed_slice();
        let snapshot = Region::remap(0, &GHEAP.alloc(content.len() as u64, 1).unwrap());
        BytesAccess::write(snapshot.deref(), &0, &content).unwrap();
        for _ in 0..num_queues {
            virtio_device.add_queue(Queue::new(&memory, QueueSetting { max_queue_size: 16 }, VirtIOBlkQueue::new(memory, VirtIOBlkDiskSnapshot::new(&snapshot), virtio_device.get_irq_vec().sender(0).unwrap())));
        }
        VirtIOBlk {
            virtio_device,
        }
    }
}

impl DeviceAccess for VirtIOBlk {
    fn device(&self) -> &Device {
        &self.virtio_device
    }
}

impl MMIODevice for VirtIOBlk {}

impl BytesAccess for VirtIOBlk {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        self.write_bytes(addr, data);
        Ok(0)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        self.read_bytes(addr, data);
        Ok(0)
    }
}

impl U32Access for VirtIOBlk {
    fn write(&self, addr: &u64, data: u32) {
        MMIODevice::write(self, addr, &data)
    }

    fn read(&self, addr: &u64) -> u32 {
        MMIODevice::read(self, addr)
    }
}




