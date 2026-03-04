//! Initramfs - Initial RAM Filesystem
//!
//! Embeds a minimal filesystem in the kernel binary that is mounted at boot.

use super::inode::{
    DirEntry, FileModeKind, FilePermissions, FileType, Inode, InodeRef, Stat, VfsError, BLOCK_SIZE,
    SECTOR_SIZE,
};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

const INITRAMFS_MAGIC: u32 = 0x20206b72;
const DEFAULT_ROOT_UID: u32 = 0;
const DEFAULT_ROOT_GID: u32 = 0;
const DIR_NLINK: u32 = 2;
const FILE_NLINK: u32 = 1;

struct InitramfsHeader {
    magic: u32,
    version: u32,
    num_files: u32,
}

struct InitramfsEntry {
    name: String,
    data: Vec<u8>,
    file_type: FileType,
}

pub struct InitramfsInode {
    inode_type: FileType,
    permissions: FilePermissions,
    stat: Stat,
    name: String,
    parent: Option<InodeRef>,
    children: Vec<InodeRef>,
    data: Vec<u8>,
}

static INODE_COUNTER: Mutex<u64> = Mutex::new(1);

impl InitramfsInode {
    pub fn new(name: &str, inode_type: FileType) -> Self {
        let mut counter = INODE_COUNTER.lock();
        let ino = *counter;
        *counter += 1;

        let mode = match inode_type {
            FileType::Directory => FileModeKind::Directory.bits(),
            _ => FileModeKind::RegularFile.bits(),
        };
        let nlink = if inode_type == FileType::Directory {
            DIR_NLINK
        } else {
            FILE_NLINK
        };

        Self {
            inode_type,
            permissions: if inode_type == FileType::Directory {
                FilePermissions::default_directory()
            } else {
                FilePermissions::default_file()
            },
            stat: Stat {
                st_ino: ino,
                st_mode: mode,
                st_nlink: nlink,
                st_uid: DEFAULT_ROOT_UID,
                st_gid: DEFAULT_ROOT_GID,
                st_size: 0,
                st_blksize: BLOCK_SIZE,
                st_blocks: 0,
            },
            name: name.to_string(),
            parent: None,
            children: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
        self.stat.st_size = self.data.len() as u64;
        self.stat.st_blocks = (self.stat.st_size + SECTOR_SIZE - 1) / SECTOR_SIZE;
    }
}

impl Inode for InitramfsInode {
    fn inode_type(&self) -> FileType {
        self.inode_type
    }

    fn permissions(&self) -> FilePermissions {
        self.permissions
    }

    fn stat(&self) -> Stat {
        self.stat
    }

    fn set_permissions(&mut self, perms: FilePermissions) {
        self.permissions = perms;
    }

    fn set_owner(&mut self, uid: u32, gid: u32) {
        self.stat.st_uid = uid;
        self.stat.st_gid = gid;
    }

    fn set_size(&mut self, size: u64) {
        self.stat.st_size = size;
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<InodeRef> {
        self.parent.clone()
    }

    fn set_parent(&mut self, parent: Option<InodeRef>) {
        self.parent = parent;
    }

    fn lookup(&self, name: &str) -> Option<InodeRef> {
        if self.inode_type != FileType::Directory {
            return None;
        }
        for child in &self.children {
            let child_lock = child.lock();
            if child_lock.name() == name {
                return Some(child.clone());
            }
        }
        None
    }

    fn mkdir(&mut self, _name: &str, _perms: FilePermissions) -> Option<InodeRef> {
        None
    }

    fn create(&mut self, _name: &str, _perms: FilePermissions) -> Option<InodeRef> {
        None
    }

    fn unlink(&mut self, _name: &str) -> Result<(), VfsError> {
        Err(VfsError::OperationNotSupported)
    }

    fn rmdir(&mut self, _name: &str) -> Result<(), VfsError> {
        Err(VfsError::OperationNotSupported)
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, VfsError> {
        if self.inode_type == FileType::Directory {
            return Err(VfsError::IsDirectory);
        }

        let start = offset as usize;
        if start >= self.data.len() {
            return Ok(0);
        }

        let end = (start + buf.len()).min(self.data.len());
        let copy_len = end - start;

        buf[..copy_len].copy_from_slice(&self.data[start..end]);
        Ok(copy_len)
    }

    fn write(&mut self, _offset: u64, _buf: &[u8]) -> Result<usize, VfsError> {
        Err(VfsError::OperationNotSupported)
    }

    fn readdir(&self, offset: u64) -> Option<DirEntry> {
        if self.inode_type != FileType::Directory {
            return None;
        }

        let idx = offset as usize;
        if idx >= self.children.len() {
            return None;
        }

        let child = &self.children[idx];
        let child_lock = child.lock();

        Some(DirEntry {
            inode: child_lock.stat().st_ino,
            offset: offset + 1,
            name: child_lock.name().to_string(),
            file_type: child_lock.inode_type(),
        })
    }

    fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    fn truncate(&mut self, _size: u64) -> Result<(), VfsError> {
        Err(VfsError::OperationNotSupported)
    }
}

pub fn create_initramfs_root() -> InodeRef {
    let mut root = InitramfsInode::new("/", FileType::Directory);

    let mut bin_dir = InitramfsInode::new("bin", FileType::Directory);
    let mut bin_sh = InitramfsInode::new("sh", FileType::RegularFile);
    bin_sh.set_data(b"#!/bin/sh\necho 'Simple shell'\n".to_vec());
    let bin_sh_ref: InodeRef = Arc::new(Mutex::new(bin_sh));
    bin_dir.children.push(bin_sh_ref);

    let mut etc_dir = InitramfsInode::new("etc", FileType::Directory);
    let mut hostname = InitramfsInode::new("hostname", FileType::RegularFile);
    hostname.set_data(b"rustkernel\n".to_vec());
    let hostname_ref: InodeRef = Arc::new(Mutex::new(hostname));
    etc_dir.children.push(hostname_ref);

    let mut init = InitramfsInode::new("init", FileType::RegularFile);
    init.set_data(b"#!/bin/sh\necho 'Welcome to Rust Kernel!'\n".to_vec());
    let init_ref: InodeRef = Arc::new(Mutex::new(init));

    let mut hello = InitramfsInode::new("hello.txt", FileType::RegularFile);
    hello.set_data(b"Hello from Rust Kernel!\n".to_vec());
    let hello_ref: InodeRef = Arc::new(Mutex::new(hello));

    let bin_ref: InodeRef = Arc::new(Mutex::new(bin_dir));
    let etc_ref: InodeRef = Arc::new(Mutex::new(etc_dir));

    root.children.push(init_ref);
    root.children.push(hello_ref);
    root.children.push(bin_ref);
    root.children.push(etc_ref);

    Arc::new(Mutex::new(root))
}
