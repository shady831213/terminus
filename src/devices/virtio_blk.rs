use terminus_spaceport::memory::region::Region;
use terminus_spaceport::memory::prelude::*;
use std::rc::Rc;
use terminus_spaceport::virtio::{QueueClient, Queue, Result, Error, DESC_F_WRITE, DescMeta};
use std::ops::Deref;

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;
//SECTOR_SIZE= 512
const VIRTIO_BLK_SECTOR_SHIFT: u32 = 9;

const VIRTIO_BLK_S_OK: u8 = 0;

#[derive(Default)]
struct VirtIOBlkHeader {
    ty: u32,
    ioprio: u32,
    sector_num: u32,
}

struct VirtIOBlkQueue {
    memory: Rc<Region>,
    disk: Rc<Region>,
}

impl VirtIOBlkQueue {
    fn new(memory: &Rc<Region>, disk: &Rc<Region>) -> VirtIOBlkQueue {
        VirtIOBlkQueue {
            memory: memory.clone(),
            disk: disk.clone(),
        }
    }
}

impl QueueClient for VirtIOBlkQueue {
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
                BytesAccess::read(self.memory.deref(), &desc.addr, &mut buffer[offset..next_offset]);
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
                BytesAccess::read(self.disk.deref(), &disk_offset, &mut read_buffer[..read_len as usize - 1]);
                read_buffer[read_len as usize - 1] = VIRTIO_BLK_S_OK;
                let mut offset: usize = 0;
                for desc in read_descs.iter() {
                    let next_offset = offset + desc.len as usize;
                    BytesAccess::write(self.memory.deref(), &desc.addr, &read_buffer[offset..next_offset]);
                    offset = next_offset;
                }
                queue.set_used(desc_head, read_len)?;
            }
            VIRTIO_BLK_T_OUT => {
                BytesAccess::write(self.disk.deref(), &disk_offset, &write_buffer[header_size..]);
                U8Access::write(self.memory.deref(), &read_descs.first().unwrap().addr, VIRTIO_BLK_S_OK);
                queue.set_used(desc_head, 1)?;
            }
            _ => return Err(Error::ClientError(format!("invalid block ty {:#x}!", header.ty)))
        }
        queue.update_last_avail();
        Ok(true)
    }
}