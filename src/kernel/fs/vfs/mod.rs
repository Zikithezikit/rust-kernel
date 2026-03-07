//! Virtual File System (VFS) module
//!
//! This module provides a virtual file system layer similar to Linux VFS:
//! - Inode interface for files and directories
//! - File abstraction
//! - Mount points and namespace
//! - Dentry cache (Linux-like)
//! - Super block (Linux-like)
//! - Inode cache and implementations

pub mod block_dev;
pub mod dentry;
pub mod file;
pub mod initramfs;
pub mod inode;
pub mod inode_impl;
pub mod mount;
pub mod superblock;
pub mod tmpfs;

use crate::fs::vfs::inode::InodeRef;
use crate::include::error::KernelResult;
use alloc::sync::Arc;
use spin::Mutex;

pub fn init() -> KernelResult<()> {
    let root = initramfs::create_initramfs_root();
    mount::init_mount_namespace(root.clone());

    // Set root in kernel instance
    let kernel_ref = crate::kernel_instance::kernel();
    kernel_ref.set_mount_ns(mount::MountNamespace::new(root));

    // Mount ATA devices if present
    if let Some(dev) = crate::drivers::ata::get_primary_master() {
        let hda = block_dev::BlockDeviceInode::new("hda", dev);
        let hda_ref: InodeRef = Arc::new(Mutex::new(hda));

        if let Some(ns) = kernel_ref.mount_ns_mut() {
            if let Ok(dev_dir) = ns.walk_path("/dev") {
                let mut dev_dir_lock = dev_dir.lock();
                let _ = dev_dir_lock.link(hda_ref, "hda");
            }
        }
    }

    Ok(())
}
