//! Tmpfs - Temporary Memory Filesystem
//!
//! An in-memory filesystem that stores files in RAM.

use super::inode::{
    DirEntry, FileModeKind, FilePermissions, FileType, Inode, InodeRef, Stat, VfsError, BLOCK_SIZE,
    SECTOR_SIZE,
};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

const DEFAULT_ROOT_UID: u32 = 0;
const DEFAULT_ROOT_GID: u32 = 0;
const DIR_NLINK: u32 = 2;
const FILE_NLINK: u32 = 1;

pub struct TmpfsInode {
    inode_type: FileType,
    permissions: FilePermissions,
    stat: Stat,
    name: String,
    parent: Option<InodeRef>,
    children: Vec<InodeRef>,
    data: Vec<u8>,
}

static INODE_COUNTER: Mutex<u64> = Mutex::new(1);

impl TmpfsInode {
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

impl Inode for TmpfsInode {
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

    fn mkdir(&mut self, name: &str, perms: FilePermissions) -> Option<InodeRef> {
        if self.inode_type != FileType::Directory {
            return None;
        }

        if self.lookup(name).is_some() {
            return None;
        }

        let mut child_inode = TmpfsInode::new(name, FileType::Directory);
        child_inode.set_permissions(perms);

        let child_ref: InodeRef = Arc::new(Mutex::new(child_inode));
        self.children.push(child_ref.clone());

        Some(child_ref)
    }

    fn create(&mut self, name: &str, perms: FilePermissions) -> Option<InodeRef> {
        if self.inode_type != FileType::Directory {
            return None;
        }

        if self.lookup(name).is_some() {
            return None;
        }

        let mut file_inode = TmpfsInode::new(name, FileType::RegularFile);
        file_inode.set_permissions(perms);

        let file_ref: InodeRef = Arc::new(Mutex::new(file_inode));
        self.children.push(file_ref.clone());

        Some(file_ref)
    }

    fn unlink(&mut self, name: &str) -> Result<(), VfsError> {
        if self.inode_type != FileType::Directory {
            return Err(VfsError::NotADirectory);
        }

        let pos = self
            .children
            .iter()
            .position(|c| c.lock().name() == name)
            .ok_or(VfsError::NotFound)?;

        let child = &self.children[pos];
        if child.lock().inode_type() == FileType::Directory {
            return Err(VfsError::IsDirectory);
        }

        self.children.remove(pos);
        Ok(())
    }

    fn rmdir(&mut self, name: &str) -> Result<(), VfsError> {
        if self.inode_type != FileType::Directory {
            return Err(VfsError::NotADirectory);
        }

        let pos = self
            .children
            .iter()
            .position(|c| c.lock().name() == name)
            .ok_or(VfsError::NotFound)?;

        let child = &self.children[pos];
        let child_lock = child.lock();

        if child_lock.inode_type() != FileType::Directory {
            return Err(VfsError::NotADirectory);
        }

        if !child_lock.is_empty() {
            return Err(VfsError::NotEmpty);
        }

        drop(child_lock);
        self.children.remove(pos);
        Ok(())
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

    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<usize, VfsError> {
        if self.inode_type == FileType::Directory {
            return Err(VfsError::IsDirectory);
        }

        let start = offset as usize;

        if start + buf.len() > self.data.len() {
            self.data.resize(start + buf.len(), 0);
        }

        self.data[start..start + buf.len()].copy_from_slice(buf);
        self.stat.st_size = self.data.len() as u64;
        self.stat.st_blocks = (self.stat.st_size + SECTOR_SIZE - 1) / SECTOR_SIZE;

        Ok(buf.len())
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

    fn truncate(&mut self, size: u64) -> Result<(), VfsError> {
        self.data.resize(size as usize, 0);
        self.stat.st_size = size;
        self.stat.st_blocks = (size + SECTOR_SIZE - 1) / SECTOR_SIZE;
        Ok(())
    }

    fn symlink(&mut self, target: &str, name: &str) -> Result<(), VfsError> {
        if self.inode_type != FileType::Directory {
            return Err(VfsError::NotADirectory);
        }

        if self.lookup(name).is_some() {
            return Err(VfsError::Exists);
        }

        let mut link_inode = TmpfsInode::new(name, FileType::Symlink);
        link_inode.set_data(target.as_bytes().to_vec());
        link_inode.stat.st_mode = FileModeKind::Symlink.bits();

        let link_ref: InodeRef = Arc::new(Mutex::new(link_inode));
        self.children.push(link_ref);
        Ok(())
    }

    fn readlink(&self) -> Option<String> {
        if self.inode_type != FileType::Symlink {
            return None;
        }
        String::from_utf8(self.data.clone()).ok()
    }
}

pub fn create_tmpfs_root() -> InodeRef {
    let mut root = TmpfsInode::new("/", FileType::Directory);

    let tmp_dir = TmpfsInode::new("tmp", FileType::Directory);
    let var_dir = TmpfsInode::new("var", FileType::Directory);
    let dev_dir = TmpfsInode::new("dev", FileType::Directory);
    let proc_dir = TmpfsInode::new("proc", FileType::Directory);

    let tmp_ref: InodeRef = Arc::new(Mutex::new(tmp_dir));
    let var_ref: InodeRef = Arc::new(Mutex::new(var_dir));
    let dev_ref: InodeRef = Arc::new(Mutex::new(dev_dir));
    let proc_ref: InodeRef = Arc::new(Mutex::new(proc_dir));

    root.children.push(tmp_ref);
    root.children.push(var_ref);
    root.children.push(dev_ref);
    root.children.push(proc_ref);

    Arc::new(Mutex::new(root))
}
