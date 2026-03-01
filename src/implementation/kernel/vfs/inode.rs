//! VFS Inode interface
//!
//! Defines the inode abstraction - the core metadata object in VFS

use alloc::string::ToString;
use alloc::sync::Arc;
use spin::Mutex;

pub const SECTOR_SIZE: u64 = 512;
pub const BLOCK_SIZE: u32 = 4096;

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
pub enum FileModeKind {
    RegularFile,
    Directory,
    Symlink,
}

impl FileModeKind {
    pub const fn bits(self) -> u16 {
        match self {
            FileModeKind::RegularFile => 0o100644,
            FileModeKind::Directory => 0o40755,
            FileModeKind::Symlink => 0o120777,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileModeBit {
    OwnerRead,
    OwnerWrite,
    OwnerExec,
    GroupRead,
    GroupWrite,
    GroupExec,
    OtherRead,
    OtherWrite,
    OtherExec,
}

impl FileModeBit {
    pub const fn bit(self) -> u16 {
        match self {
            FileModeBit::OwnerRead => 0o400,
            FileModeBit::OwnerWrite => 0o200,
            FileModeBit::OwnerExec => 0o100,
            FileModeBit::GroupRead => 0o040,
            FileModeBit::GroupWrite => 0o020,
            FileModeBit::GroupExec => 0o010,
            FileModeBit::OtherRead => 0o004,
            FileModeBit::OtherWrite => 0o002,
            FileModeBit::OtherExec => 0o001,
        }
    }
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
            mode |= FileModeBit::OwnerRead.bit();
        }
        if perms.owner_write {
            mode |= FileModeBit::OwnerWrite.bit();
        }
        if perms.owner_exec {
            mode |= FileModeBit::OwnerExec.bit();
        }
        if perms.group_read {
            mode |= FileModeBit::GroupRead.bit();
        }
        if perms.group_write {
            mode |= FileModeBit::GroupWrite.bit();
        }
        if perms.group_exec {
            mode |= FileModeBit::GroupExec.bit();
        }
        if perms.other_read {
            mode |= FileModeBit::OtherRead.bit();
        }
        if perms.other_write {
            mode |= FileModeBit::OtherWrite.bit();
        }
        if perms.other_exec {
            mode |= FileModeBit::OtherExec.bit();
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
    fn is_empty(&self) -> bool {
        false
    }
    fn truncate(&mut self, _size: u64) -> Result<(), VfsError> {
        Err(VfsError::OperationNotSupported)
    }
    fn sync(&self) -> Result<(), VfsError> {
        Ok(())
    }
    fn chmod(&mut self, mode: FileMode) -> Result<(), VfsError> {
        let perms_with_mode = FilePermissions {
            owner_read: mode.0 & FileModeBit::OwnerRead.bit() != 0,
            owner_write: mode.0 & FileModeBit::OwnerWrite.bit() != 0,
            owner_exec: mode.0 & FileModeBit::OwnerExec.bit() != 0,
            group_read: mode.0 & FileModeBit::GroupRead.bit() != 0,
            group_write: mode.0 & FileModeBit::GroupWrite.bit() != 0,
            group_exec: mode.0 & FileModeBit::GroupExec.bit() != 0,
            other_read: mode.0 & FileModeBit::OtherRead.bit() != 0,
            other_write: mode.0 & FileModeBit::OtherWrite.bit() != 0,
            other_exec: mode.0 & FileModeBit::OtherExec.bit() != 0,
        };
        self.set_permissions(perms_with_mode);
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
    InvalidSeek,
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
            VfsError::InvalidSeek => write!(f, "Invalid seek position"),
            VfsError::OperationNotSupported => write!(f, "Operation not supported"),
            VfsError::IoError => write!(f, "I/O error"),
            VfsError::Busy => write!(f, "Resource busy"),
        }
    }
}
