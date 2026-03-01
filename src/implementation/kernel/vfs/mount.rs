//! VFS Mount namespace
//!
//! Provides mount point management for the virtual file system

use super::inode::{FileType, InodeRef, VfsError};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountFlags {
    pub read_only: bool,
    pub no_exec: bool,
    pub no_suid: bool,
    pub no_dev: bool,
    pub synchronous: bool,
    pub remount: bool,
    pub bind: bool,
    pub move_flag: bool,
}

impl MountFlags {
    pub const fn empty() -> Self {
        Self {
            read_only: false,
            no_exec: false,
            no_suid: false,
            no_dev: false,
            synchronous: false,
            remount: false,
            bind: false,
            move_flag: false,
        }
    }

    pub const fn rdonly() -> Self {
        Self {
            read_only: true,
            ..Self::empty()
        }
    }
}

pub struct MountPoint {
    pub path: String,
    pub device: Option<InodeRef>,
    pub fs_root: InodeRef,
    pub flags: MountFlags,
}

impl MountPoint {
    pub fn new(path: &str, fs_root: InodeRef) -> Self {
        Self {
            path: path.to_string(),
            device: None,
            fs_root,
            flags: MountFlags::empty(),
        }
    }

    pub fn with_device(path: &str, device: InodeRef, fs_root: InodeRef) -> Self {
        Self {
            path: path.to_string(),
            device: Some(device),
            fs_root,
            flags: MountFlags::empty(),
        }
    }

    pub fn is_readonly(&self) -> bool {
        self.flags.read_only
    }
}

pub struct MountNamespace {
    mounts: Vec<MountPoint>,
    root: InodeRef,
}

impl MountNamespace {
    pub fn new(root: InodeRef) -> Self {
        Self {
            mounts: Vec::new(),
            root,
        }
    }

    pub fn mount(
        &mut self,
        path: &str,
        device: Option<InodeRef>,
        fs_root: InodeRef,
    ) -> Result<(), VfsError> {
        if path.is_empty() {
            return Err(VfsError::InvalidFilename);
        }

        if self.find_mount(path).is_some() {
            return Err(VfsError::Exists);
        }

        let mount_point = match device {
            Some(dev) => MountPoint::with_device(path, dev, fs_root),
            None => MountPoint::new(path, fs_root),
        };

        self.mounts.push(mount_point);
        Ok(())
    }

    pub fn unmount(&mut self, path: &str) -> Result<(), VfsError> {
        let idx = self
            .mounts
            .iter()
            .position(|m| m.path == path)
            .ok_or(VfsError::NotFound)?;

        if self.mounts[idx].is_readonly() {
            return Err(VfsError::Busy);
        }

        self.mounts.remove(idx);
        Ok(())
    }

    pub fn remount(&mut self, path: &str, flags: MountFlags) -> Result<(), VfsError> {
        let mount = self
            .mounts
            .iter_mut()
            .find(|m| m.path == path)
            .ok_or(VfsError::NotFound)?;

        mount.flags = flags;
        Ok(())
    }

    pub fn find_mount(&self, path: &str) -> Option<&MountPoint> {
        let mut longest_match: Option<&MountPoint> = None;

        for mount in &self.mounts {
            if path.starts_with(&mount.path) {
                match longest_match {
                    None => longest_match = Some(mount),
                    Some(current) => {
                        if mount.path.len() > current.path.len() {
                            longest_match = Some(mount);
                        }
                    }
                }
            }
        }

        longest_match
    }

    pub fn get_root(&self) -> &InodeRef {
        &self.root
    }

    pub fn set_root(&mut self, root: InodeRef) {
        self.root = root;
    }

    pub fn get_mounts(&self) -> &[MountPoint] {
        &self.mounts
    }

    pub fn walk_path(&self, path: &str) -> Result<InodeRef, VfsError> {
        let path = path.trim_start_matches('/');

        if path.is_empty() {
            return Ok(self.root.clone());
        }

        let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let mut current = if let Some(mount) = self.find_mount(path) {
            mount.fs_root.clone()
        } else {
            self.root.clone()
        };

        for component in components {
            let (name, file_type, parent, next) = {
                let inode = current.lock();
                (
                    inode.name().to_string(),
                    inode.inode_type(),
                    inode.parent(),
                    inode.lookup(component),
                )
            };

            if name == component && file_type == FileType::Directory {
                if let Some(p) = parent {
                    current = p;
                }
            }

            if let Some(n) = next {
                current = n;
            } else {
                return Err(VfsError::NotFound);
            }
        }

        Ok(current)
    }
}

pub static MOUNT_NAMESPACE: Mutex<Option<MountNamespace>> = Mutex::new(None);

pub fn init_mount_namespace(root: InodeRef) {
    let mut ns = MOUNT_NAMESPACE.lock();
    *ns = Some(MountNamespace::new(root));
}

pub fn get_mount_namespace() -> spin::MutexGuard<'static, Option<MountNamespace>> {
    MOUNT_NAMESPACE.lock()
}
