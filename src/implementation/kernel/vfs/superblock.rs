//! VFS super_block - Filesystem metadata (Linux-like)
//!
//! In Linux, super_block represents a mounted filesystem

use super::inode::InodeRef;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuperBlockFlags {
    pub read_only: bool,
    pub no_exec: bool,
    pub no_suid: bool,
    pub no_dev: bool,
    pub synchronous: bool,
    pub remount: bool,
}

impl SuperBlockFlags {
    pub const fn empty() -> Self {
        Self {
            read_only: false,
            no_exec: false,
            no_suid: false,
            no_dev: false,
            synchronous: false,
            remount: false,
        }
    }

    pub const fn rdonly() -> Self {
        Self {
            read_only: true,
            ..Self::empty()
        }
    }
}

#[allow(dead_code)]
pub struct SuperBlock {
    pub s_dev: u64,
    pub s_inodes: Vec<InodeRef>,
    pub s_root: InodeRef,
    pub s_flags: SuperBlockFlags,
    pub s_fs_info: String,
}

#[allow(dead_code)]
impl SuperBlock {
    pub fn new(s_dev: u64, root: InodeRef, fs_info: &str) -> Self {
        Self {
            s_dev,
            s_inodes: Vec::new(),
            s_root: root,
            s_flags: SuperBlockFlags::empty(),
            s_fs_info: fs_info.to_string(),
        }
    }

    pub fn alloc_inode(&mut self, inode: InodeRef) {
        self.s_inodes.push(inode);
    }

    pub fn put_inode(&mut self, inode: InodeRef) {
        if let Some(pos) = self.s_inodes.iter().position(|i| Arc::ptr_eq(i, &inode)) {
            self.s_inodes.remove(pos);
        }
    }

    pub fn read_inode(&self, ino: u64) -> Option<InodeRef> {
        for inode_ref in &self.s_inodes {
            let inode = inode_ref.lock();
            if inode.stat().st_ino == ino {
                return Some(inode_ref.clone());
            }
        }
        None
    }
}
