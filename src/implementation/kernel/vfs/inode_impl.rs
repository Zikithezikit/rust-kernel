//! VFS inode cache and concrete inode implementations

use super::inode::{
    DirEntry, FileMode, FileModeBit, FileModeKind, FilePermissions, FileType, Inode, InodeRef,
    Stat, VfsError, BLOCK_SIZE, SECTOR_SIZE,
};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Clone)]
pub struct InodeImpl {
    pub inode_type: FileType,
    pub permissions: FilePermissions,
    pub stat: Stat,
    pub name: String,
    pub parent: Option<InodeRef>,
    pub children: Vec<InodeRef>,
    pub data: Vec<u8>,
}

static INODE_COUNTER: Mutex<u64> = Mutex::new(1);

impl InodeImpl {
    pub fn new(name: &str, inode_type: FileType) -> Self {
        let mut counter = INODE_COUNTER.lock();
        let ino = *counter;
        *counter += 1;

        Self {
            inode_type,
            permissions: if inode_type == FileType::Directory {
                FilePermissions::default_directory()
            } else {
                FilePermissions::default_file()
            },
            stat: Stat {
                st_ino: ino,
                st_mode: if inode_type == FileType::Directory {
                    FileModeKind::Directory.bits()
                } else {
                    FileModeKind::RegularFile.bits()
                },
                st_nlink: if inode_type == FileType::Directory {
                    2
                } else {
                    1
                },
                st_uid: 0,
                st_gid: 0,
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

impl Inode for InodeImpl {
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

        let mut child_inode = InodeImpl::new(name, FileType::Directory);
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

        let mut file_inode = InodeImpl::new(name, FileType::RegularFile);
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

        let mut link_inode = InodeImpl::new(name, FileType::Symlink);
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
