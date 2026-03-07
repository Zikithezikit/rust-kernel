//! Block device inode implementation
//!
//! Provides an Inode wrapper around a BlockDevice, allowing the VFS to interact
//! with disks and partitions.

use super::inode::{FilePermissions, FileType, Inode, InodeRef, Stat, VfsError};
use crate::drivers::block::BlockDevice;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use spin::Mutex;

pub struct BlockDeviceInode {
    pub device: Arc<Mutex<dyn BlockDevice>>,
    pub name: String,
    pub stat: Stat,
    pub parent: Option<InodeRef>,
}

impl BlockDeviceInode {
    pub fn new(name: &str, device: Arc<Mutex<dyn BlockDevice>>) -> Self {
        let (num_sectors, sector_size) = {
            let d = device.lock();
            (d.num_sectors(), d.sector_size())
        };
        let size = num_sectors * sector_size as u64;

        Self {
            device,
            name: name.to_string(),
            stat: Stat {
                st_ino: 0,         // Should be assigned by a proper inode allocator
                st_mode: 0o060644, // Block device, rw-r--r--
                st_nlink: 1,
                st_uid: 0,
                st_gid: 0,
                st_size: size,
                st_blksize: sector_size as u32,
                st_blocks: num_sectors,
            },
            parent: None,
        }
    }
}

impl Inode for BlockDeviceInode {
    fn inode_type(&self) -> FileType {
        FileType::BlockDevice
    }

    fn permissions(&self) -> FilePermissions {
        FilePermissions::default_file()
    }

    fn stat(&self) -> Stat {
        self.stat
    }

    fn set_permissions(&mut self, _perms: FilePermissions) {}

    fn set_owner(&mut self, uid: u32, gid: u32) {
        self.stat.st_uid = uid;
        self.stat.st_gid = gid;
    }

    fn set_size(&mut self, _size: u64) {}

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<InodeRef> {
        self.parent.clone()
    }

    fn set_parent(&mut self, parent: Option<InodeRef>) {
        self.parent = parent;
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, VfsError> {
        let mut device = self.device.lock();
        let sector_size = device.sector_size() as u64;
        let start_sector = offset / sector_size;
        let sector_offset = (offset % sector_size) as usize;

        if sector_offset != 0 {
            return Err(VfsError::OperationNotSupported);
        }

        let count = (buf.len() + sector_size as usize - 1) / sector_size as usize;

        match device.read_sectors(start_sector, count, buf) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(VfsError::IoError),
        }
    }

    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<usize, VfsError> {
        let mut device = self.device.lock();
        let sector_size = device.sector_size() as u64;
        let start_sector = offset / sector_size;
        let sector_offset = (offset % sector_size) as usize;

        if sector_offset != 0 {
            return Err(VfsError::OperationNotSupported);
        }

        let count = (buf.len() + sector_size as usize - 1) / sector_size as usize;

        match device.write_sectors(start_sector, count, buf) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(VfsError::IoError),
        }
    }
}
