//! VFS dentry - Directory entry (Linux-like)
//!
//! In Linux, dentry links inode names to inodes and is cached in the dentry cache

use super::inode::InodeRef;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DentryFlags {
    None,
    Negative,
    Mounted,
}

#[derive(Clone)]
pub struct Dentry {
    pub d_name: String,
    pub d_inode: Option<InodeRef>,
    pub d_parent: Option<Arc<Mutex<Dentry>>>,
    pub d_child: Vec<Arc<Mutex<Dentry>>>,
    pub d_flags: DentryFlags,
    pub d_count: usize,
}

impl Dentry {
    pub fn new(name: &str, inode: Option<InodeRef>, parent: Option<Arc<Mutex<Dentry>>>) -> Self {
        let is_negative = inode.is_none();
        Self {
            d_name: name.to_string(),
            d_inode: inode,
            d_parent: parent,
            d_child: Vec::new(),
            d_flags: if is_negative {
                DentryFlags::Negative
            } else {
                DentryFlags::None
            },
            d_count: 0,
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Arc<Mutex<Dentry>>> {
        for child in &self.d_child {
            let child_lock = child.lock();
            if child_lock.d_name == name {
                return Some(child.clone());
            }
        }
        None
    }

    pub fn add_child(&mut self, child: Arc<Mutex<Dentry>>) {
        self.d_child.push(child);
    }

    pub fn remove_child(&mut self, name: &str) {
        if let Some(pos) = self.d_child.iter().position(|c| c.lock().d_name == name) {
            self.d_child.remove(pos);
        }
    }

    pub fn d_is_positive(&self) -> bool {
        self.d_inode.is_some()
    }

    pub fn d_is_negative(&self) -> bool {
        self.d_flags == DentryFlags::Negative
    }

    pub fn d_mountpoint(&self) -> bool {
        self.d_flags == DentryFlags::Mounted
    }

    pub fn d_invalidate(&mut self) {
        self.d_inode = None;
        self.d_flags = DentryFlags::Negative;
    }
}
