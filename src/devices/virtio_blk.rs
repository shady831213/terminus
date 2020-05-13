use terminus_spaceport::memory::region::{Region, GHEAP};
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;
use terminus_spaceport::virtio::{QueueClient, Queue, Result, Error, DESC_F_WRITE, DescMeta, Device, QueueSetting, DeviceAccess, MMIODevice};
use std::ops::Deref;
use terminus_spaceport::irq::IrqVecSender;
use std::fs;
use std::fs::{File, OpenOptions};
use std::os::unix::prelude::FileExt;

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;
//SECTOR_SIZE= 512
const VIRTIO_BLK_SECTOR_SHIFT: u64 = 9;

const VIRTIO_BLK_S_OK: u8 = 0;
const VIRTIO_BLK_S_IOERR: u8 = 1;

#[derive(Default, Debug)]
#[repr(C)]
struct VirtIOBlkHeader {
    ty: u32,
    ioprio: u32,
    sector_num: u64,
}

pub enum VirtIOBlkConfig {
    RO,
    RW,
    SNAPSHOT,
}

impl VirtIOBlkConfig {
    pub fn new(val: &str) -> VirtIOBlkConfig {
        match val {
            "ro" => VirtIOBlkConfig::RO,
            "rw" => VirtIOBlkConfig::RW,
            _ => VirtIOBlkConfig::SNAPSHOT
        }
    }
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

struct VirtIOBlkFile {
    fp: Rc<File>
}

impl VirtIOBlkFile {
    fn new(fp: &Rc<File>) -> VirtIOBlkFile {
        VirtIOBlkFile { fp: fp.clone() }
    }
}

impl BytesAccess for VirtIOBlkFile {
    fn write(&self, addr: &u64, data: &[u8]) -> std::result::Result<usize, String> {
        if self.fp.write_all_at(data, *addr).is_err() {
            Err("write err".to_string())
        } else if self.fp.sync_all().is_err() {
            Err("sync err".to_string())
        } else {
            Ok(data.len())
        }
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> std::result::Result<usize, String> {
        if self.fp.sync_all().is_err() {
            Err("sync err".to_string())
        } else if self.fp.read_exact_at(data, *addr).is_err() {
            Err("read err".to_string())
        } else {
            Ok(data.len())
        }
    }
}

struct VirtIOBlkQueue<T: BytesAccess> {
    memory: Rc<Region>,
    disk: T,
    irq_sender: IrqVecSender,
}

impl<T: BytesAccess> VirtIOBlkQueue<T> {
    fn new(memory: &Rc<Region>, disk: T, irq_sender: IrqVecSender) -> VirtIOBlkQueue<T> {
        VirtIOBlkQueue {
            memory: memory.clone(),
            disk,
            irq_sender,
        }
    }
}

impl<T: BytesAccess> QueueClient for VirtIOBlkQueue<T> {
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
        if write_len as usize >= header_size {
            unsafe { std::slice::from_raw_parts_mut((&mut header as *mut VirtIOBlkHeader) as *mut u8, header_size).copy_from_slice(&write_buffer[..header_size]) }
        } else {
            return Err(Error::ClientError("invalid block header!".to_string()));
        }

        let disk_offset = header.sector_num << VIRTIO_BLK_SECTOR_SHIFT;

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
    num_sectors: u64,
}

impl VirtIOBlk {
    pub fn new(memory: &Rc<Region>, irq_sender: IrqVecSender, num_queues: usize, file_name: &str, config: VirtIOBlkConfig) -> VirtIOBlk {
        assert!(num_queues > 0);
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            1,
                                            2, 0, 0,
        );
        virtio_device.get_irq_vec().set_enable_uncheck(0, true);
        let len = match config {
            VirtIOBlkConfig::RO => {
                let file = Rc::new(OpenOptions::new().read(true).open(file_name).expect(&format!("can not open {}!", file_name)));
                for _ in 0..num_queues {
                    // virtio_device.add_queue(Queue::new(&memory, QueueSetting { max_queue_size: 16 }, VirtIOBlkQueue::new(memory, VirtIOBlkDiskSnapshot::new(&snapshot), virtio_device.get_irq_vec().sender(0).unwrap())));
                    virtio_device.add_queue(Queue::new(&memory, QueueSetting { max_queue_size: 16 }, VirtIOBlkQueue::new(memory, VirtIOBlkFile::new(&file), virtio_device.get_irq_vec().sender(0).unwrap())));
                }
                file.metadata().unwrap().len()
            }
            VirtIOBlkConfig::RW => {
                let file = Rc::new(OpenOptions::new().read(true).write(true).create(true).open(file_name).expect(&format!("can not open {}!", file_name)));
                for _ in 0..num_queues {
                    // virtio_device.add_queue(Queue::new(&memory, QueueSetting { max_queue_size: 16 }, VirtIOBlkQueue::new(memory, VirtIOBlkDiskSnapshot::new(&snapshot), virtio_device.get_irq_vec().sender(0).unwrap())));
                    virtio_device.add_queue(Queue::new(&memory, QueueSetting { max_queue_size: 16 }, VirtIOBlkQueue::new(memory, VirtIOBlkFile::new(&file), virtio_device.get_irq_vec().sender(0).unwrap())));
                }
                file.metadata().unwrap().len()
            }
            VirtIOBlkConfig::SNAPSHOT => {
                let content = fs::read(file_name).unwrap().into_boxed_slice();
                let snapshot = Region::remap(0, &GHEAP.alloc(content.len() as u64, 1).unwrap());
                BytesAccess::write(snapshot.deref(), &0, &content).unwrap();
                for _ in 0..num_queues {
                    virtio_device.add_queue(Queue::new(&memory, QueueSetting { max_queue_size: 16 }, VirtIOBlkQueue::new(memory, VirtIOBlkDiskSnapshot::new(&snapshot), virtio_device.get_irq_vec().sender(0).unwrap())));
                }
                content.len() as u64
            }
        };
        VirtIOBlk {
            virtio_device,
            num_sectors: len >> VIRTIO_BLK_SECTOR_SHIFT,
        }
    }
}

impl DeviceAccess for VirtIOBlk {
    fn device(&self) -> &Device {
        &self.virtio_device
    }
    fn config(&self, offset: u64) -> u32 {
        if offset < 8 {
            ((self.num_sectors >> (offset << 3)) & self.config_mask(&offset)) as u32
        } else {
            0
        }
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




