//! VFS Inode interface
//!
//! Defines the inode abstraction - the core metadata object in VFS

use alloc::string::ToString;
use alloc::sync::Arc;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    RegularFile,
    Directory,
    CharacterDevice,
    BlockDevice,
    Fifo,
    Socket,
    Symlink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePermissions {
    pub owner_read: bool,
    pub owner_write: bool,
    pub owner_exec: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub group_exec: bool,
    pub other_read: bool,
    pub other_write: bool,
    pub other_exec: bool,
}

impl FilePermissions {
    pub const fn default_file() -> Self {
        Self {
            owner_read: true,
            owner_write: true,
            owner_exec: false,
            group_read: true,
            group_write: false,
            group_exec: false,
            other_read: true,
            other_write: false,
            other_exec: false,
        }
    }

    pub const fn default_directory() -> Self {
        Self {
            owner_read: true,
            owner_write: true,
            owner_exec: true,
            group_read: true,
            group_write: false,
            group_exec: true,
            other_read: true,
            other_write: false,
            other_exec: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(pub u16);

impl FileMode {
    pub const fn from_permissions(perms: &FilePermissions) -> Self {
        let mut mode: u16 = 0;
        if perms.owner_read {
            mode |= 0o400;
        }
        if perms.owner_write {
            mode |= 0o200;
        }
        if perms.owner_exec {
            mode |= 0o100;
        }
        if perms.group_read {
            mode |= 0o040;
        }
        if perms.group_write {
            mode |= 0o020;
        }
        if perms.group_exec {
            mode |= 0o010;
        }
        if perms.other_read {
            mode |= 0o004;
        }
        if perms.other_write {
            mode |= 0o002;
        }
        if perms.other_exec {
            mode |= 0o001;
        }
        Self(mode)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stat {
    pub st_ino: u64,
    pub st_mode: u16,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_size: u64,
    pub st_blksize: u32,
    pub st_blocks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub trait Inode: Send + Sync {
    fn inode_type(&self) -> FileType;
    fn permissions(&self) -> FilePermissions;
    fn stat(&self) -> Stat;
    fn set_permissions(&mut self, perms: FilePermissions);
    fn set_owner(&mut self, uid: u32, gid: u32);
    fn set_size(&mut self, size: u64);
    fn name(&self) -> &str;
    fn parent(&self) -> Option<InodeRef>;
    fn set_parent(&mut self, parent: Option<InodeRef>);
    fn lookup(&self, _name: &str) -> Option<InodeRef> {
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
    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize, VfsError> {
        Err(VfsError::OperationNotSupported)
    }
    fn write(&mut self, _offset: u64, _buf: &[u8]) -> Result<usize, VfsError> {
        Err(VfsError::OperationNotSupported)
    }
    fn readdir(&self, _offset: u64) -> Option<DirEntry> {
        None
    }
    fn truncate(&mut self, _size: u64) -> Result<(), VfsError> {
        Err(VfsError::OperationNotSupported)
    }
    fn sync(&self) -> Result<(), VfsError> {
        Ok(())
    }
    fn chmod(&mut self, mode: FileMode) -> Result<(), VfsError> {
        self.set_permissions(FilePermissions::default_file());
        let _ = mode;
        Ok(())
    }
    fn chown(&mut self, uid: u32, gid: u32) -> Result<(), VfsError> {
        self.set_owner(uid, gid);
        Ok(())
    }
    fn link(&mut self, _target: &dyn Inode, _name: &str) -> Result<(), VfsError> {
        Err(VfsError::OperationNotSupported)
    }
    fn symlink(&mut self, _target: &str, _name: &str) -> Result<(), VfsError> {
        Err(VfsError::OperationNotSupported)
    }
    fn readlink(&self) -> Option<alloc::string::String> {
        None
    }
}

pub type InodeRef = Arc<Mutex<dyn Inode>>;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub inode: u64,
    pub offset: u64,
    pub name: alloc::string::String,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsError {
    NotFound,
    PermissionDenied,
    Exists,
    NotEmpty,
    IsDirectory,
    NotADirectory,
    FileTooLarge,
    NoSpaceLeft,
    InvalidFilename,
    OperationNotSupported,
    IoError,
    Busy,
}

impl core::fmt::Display for VfsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VfsError::NotFound => write!(f, "File or directory not found"),
            VfsError::PermissionDenied => write!(f, "Permission denied"),
            VfsError::Exists => write!(f, "File or directory already exists"),
            VfsError::NotEmpty => write!(f, "Directory not empty"),
            VfsError::IsDirectory => write!(f, "Is a directory"),
            VfsError::NotADirectory => write!(f, "Not a directory"),
            VfsError::FileTooLarge => write!(f, "File too large"),
            VfsError::NoSpaceLeft => write!(f, "No space left on device"),
            VfsError::InvalidFilename => write!(f, "Invalid filename"),
            VfsError::OperationNotSupported => write!(f, "Operation not supported"),
            VfsError::IoError => write!(f, "I/O error"),
            VfsError::Busy => write!(f, "Resource busy"),
        }
    }
}
