//! VFS File abstraction
//!
//! Provides the File struct which represents an open file description

use super::inode::{FileType, InodeRef, SeekFrom, VfsError};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

pub type FileRef = Arc<Mutex<File>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileFlags {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub sync: bool,
    pub async_flag: bool,
    pub direct: bool,
    pub no_terminal: bool,
    pub truncate: bool,
    pub create: bool,
    pub exclusive: bool,
}

impl FileFlags {
    pub const fn empty() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            sync: false,
            async_flag: false,
            direct: false,
            no_terminal: false,
            truncate: false,
            create: false,
            exclusive: false,
        }
    }

    pub const fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            ..Self::empty()
        }
    }

    pub const fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            ..Self::empty()
        }
    }

    pub const fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            ..Self::empty()
        }
    }

    pub const fn append() -> Self {
        Self {
            read: true,
            write: true,
            append: true,
            ..Self::empty()
        }
    }
}

pub struct File {
    pub inode: InodeRef,
    pub offset: u64,
    pub flags: FileFlags,
    pub file_type: FileType,
}

impl File {
    pub fn new(inode: InodeRef, flags: FileFlags) -> Self {
        let file_type = {
            let inode_guard = inode.lock();
            inode_guard.inode_type()
        };
        Self {
            inode,
            offset: 0,
            flags,
            file_type,
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, VfsError> {
        if !self.flags.read {
            return Err(VfsError::PermissionDenied);
        }

        if self.file_type == FileType::Directory {
            return Err(VfsError::IsDirectory);
        }

        let inode = self.inode.clone();
        let n = inode.lock().read(self.offset, buf)?;
        self.offset += n as u64;
        Ok(n)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, VfsError> {
        if !self.flags.write {
            return Err(VfsError::PermissionDenied);
        }

        if self.file_type == FileType::Directory {
            return Err(VfsError::IsDirectory);
        }

        let mut offset = self.offset;
        if self.flags.append {
            let stat = self.inode.lock().stat();
            offset = stat.st_size;
        }

        let inode = self.inode.clone();
        let n = inode.lock().write(offset, buf)?;
        self.offset += n as u64;
        Ok(n)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64, VfsError> {
        let stat = self.inode.lock().stat();
        let file_size = stat.st_size;

        self.offset = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => {
                if offset < 0 {
                    if (-offset) as u64 > file_size {
                        return Err(VfsError::InvalidSeek);
                    }
                    file_size.saturating_sub((-offset) as u64)
                } else {
                    file_size.saturating_add(offset as u64)
                }
            }
            SeekFrom::Current(offset) => {
                if offset < 0 {
                    if (-offset) as u64 > self.offset {
                        return Err(VfsError::InvalidSeek);
                    }
                    self.offset - ((-offset) as u64)
                } else {
                    self.offset + (offset as u64)
                }
            }
        };

        Ok(self.offset)
    }

    pub fn truncate(&mut self) -> Result<(), VfsError> {
        if !self.flags.write {
            return Err(VfsError::PermissionDenied);
        }

        self.inode.lock().truncate(self.offset)
    }

    pub fn sync(&self) -> Result<(), VfsError> {
        self.inode.lock().sync()
    }

    pub fn stat(&self) -> super::inode::Stat {
        self.inode.lock().stat()
    }

    pub fn is_readable(&self) -> bool {
        self.flags.read
    }

    pub fn is_writable(&self) -> bool {
        self.flags.write
    }

    pub fn is_append(&self) -> bool {
        self.flags.append
    }

    pub fn is_directory(&self) -> bool {
        self.file_type == FileType::Directory
    }
}

pub struct FileTable {
    files: Vec<Option<FileRef>>,
    max_fd: usize,
}

impl FileTable {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            max_fd: 64,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            files: Vec::with_capacity(capacity),
            max_fd: capacity,
        }
    }

    pub fn allocate_fd(&mut self, file: FileRef) -> Option<usize> {
        for (fd, slot) in self.files.iter().enumerate() {
            if slot.is_none() {
                self.files[fd] = Some(file);
                return Some(fd);
            }
        }

        if self.files.len() < self.max_fd {
            let fd = self.files.len();
            self.files.push(Some(file));
            Some(fd)
        } else {
            None
        }
    }

    pub fn get(&self, fd: usize) -> Option<FileRef> {
        self.files.get(fd).and_then(|f| f.clone())
    }

    pub fn put(&mut self, fd: usize) -> Option<FileRef> {
        if fd < self.files.len() {
            self.files[fd].take()
        } else {
            None
        }
    }

    pub fn dup(&mut self, fd: usize) -> Option<usize> {
        let file = self.get(fd)?;
        let new_fd = self.allocate_fd(file)?;
        Some(new_fd)
    }

    pub fn dup2(&mut self, fd: usize, new_fd: usize) -> Option<usize> {
        if fd >= self.files.len() {
            return None;
        }

        let file = self.files[fd].clone()?;

        if new_fd >= self.files.len() {
            self.files.resize(new_fd + 1, None);
        }

        self.files[new_fd] = Some(file);
        Some(new_fd)
    }

    pub fn close(&mut self, fd: usize) -> Option<FileRef> {
        self.put(fd)
    }

    pub fn fcntl(&self, _fd: usize, _cmd: u64, _arg: u64) -> Result<u64, VfsError> {
        Err(VfsError::OperationNotSupported)
    }
}

impl Default for FileTable {
    fn default() -> Self {
        Self::new()
    }
}
