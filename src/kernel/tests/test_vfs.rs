use crate::drivers::serial;
use crate::fs::vfs::dentry::Dentry;
use crate::fs::vfs::file::{FileFlags, FileTable};
use crate::fs::vfs::initramfs;
use crate::fs::vfs::inode::{FilePermissions, FileType, Inode, InodeRef, SeekFrom};
use crate::fs::vfs::inode_impl::InodeImpl;
use crate::fs::vfs::mount::MountNamespace;
use crate::fs::vfs::superblock::SuperBlock;
use crate::fs::vfs::tmpfs;
use alloc::sync::Arc;
use spin::Mutex;

pub fn test_vfs_inode_creation() {
    serial::write_string("Testing VFS inode creation...\n");

    let inode = InodeImpl::new("test.txt", FileType::RegularFile);
    let inode_ref: InodeRef = Arc::new(Mutex::new(inode));

    let stat = inode_ref.lock().stat();
    let ok = stat.st_ino > 0 && inode_ref.lock().name() == "test.txt";

    serial::write_string(if ok {
        "test_vfs_inode_creation: OK\n"
    } else {
        "test_vfs_inode_creation: FAIL\n"
    });
}

pub fn test_vfs_directory_inode() {
    serial::write_string("Testing VFS directory inode...\n");

    let inode = InodeImpl::new("test_dir", FileType::Directory);
    let inode_ref: InodeRef = Arc::new(Mutex::new(inode));

    let ok = inode_ref.lock().inode_type() == FileType::Directory
        && inode_ref.lock().parent().is_none()
        && inode_ref.lock().lookup("test").is_none();

    serial::write_string(if ok {
        "test_vfs_directory_inode: OK\n"
    } else {
        "test_vfs_directory_inode: FAIL\n"
    });
}

pub fn test_vfs_inode_mkdir() {
    serial::write_string("Testing VFS inode mkdir...\n");

    let mut dir = InodeImpl::new("parent", FileType::Directory);
    let child = dir.mkdir("child", FilePermissions::default_directory());

    let ok = child.is_some() && dir.lookup("child").is_some();

    serial::write_string(if ok {
        "test_vfs_inode_mkdir: OK\n"
    } else {
        "test_vfs_inode_mkdir: FAIL\n"
    });
}

pub fn test_vfs_inode_create_file() {
    serial::write_string("Testing VFS inode create file...\n");

    let mut dir = InodeImpl::new("parent", FileType::Directory);
    let file = dir.create("file.txt", FilePermissions::default_file());

    let ok = file.is_some() && dir.lookup("file.txt").is_some();

    serial::write_string(if ok {
        "test_vfs_inode_create_file: OK\n"
    } else {
        "test_vfs_inode_create_file: FAIL\n"
    });
}

pub fn test_vfs_inode_read_write() {
    serial::write_string("Testing VFS inode read/write...\n");

    let mut inode = InodeImpl::new("test.txt", FileType::RegularFile);
    let test_data = b"Hello, VFS!";
    inode.set_data(test_data.to_vec());

    let inode_ref: InodeRef = Arc::new(Mutex::new(inode));

    let mut buf = [0u8; 64];
    let n = inode_ref.lock().read(0, &mut buf).unwrap();

    let ok = n == test_data.len() && &buf[..n] == test_data;

    serial::write_string(if ok {
        "test_vfs_inode_read_write: OK\n"
    } else {
        "test_vfs_inode_read_write: FAIL\n"
    });
}

pub fn test_vfs_inode_unlink() {
    serial::write_string("Testing VFS inode unlink...\n");

    let mut dir = InodeImpl::new("parent", FileType::Directory);
    let _ = dir.create("file.txt", FilePermissions::default_file());

    let result = dir.unlink("file.txt");
    let ok = result.is_ok() && dir.lookup("file.txt").is_none();

    serial::write_string(if ok {
        "test_vfs_inode_unlink: OK\n"
    } else {
        "test_vfs_inode_unlink: FAIL\n"
    });
}

pub fn test_vfs_inode_rmdir() {
    serial::write_string("Testing VFS inode rmdir...\n");

    let mut dir = InodeImpl::new("parent", FileType::Directory);
    let _ = dir.mkdir("child_dir", FilePermissions::default_directory());

    let result = dir.rmdir("child_dir");
    let ok = result.is_ok() && dir.lookup("child_dir").is_none();

    serial::write_string(if ok {
        "test_vfs_inode_rmdir: OK\n"
    } else {
        "test_vfs_inode_rmdir: FAIL\n"
    });
}

pub fn test_vfs_file_table() {
    serial::write_string("Testing VFS file table...\n");

    let mut table = FileTable::new();
    let inode: InodeRef = Arc::new(Mutex::new(InodeImpl::new("test", FileType::RegularFile)));
    let file = crate::fs::vfs::file::File::new(inode, FileFlags::read_write());

    let fd = table.allocate_fd(Arc::new(Mutex::new(file)));

    let ok = fd.is_some() && table.get(fd.unwrap()).is_some();

    serial::write_string(if ok {
        "test_vfs_file_table: OK\n"
    } else {
        "test_vfs_file_table: FAIL\n"
    });
}

pub fn test_vfs_file_read_write() {
    serial::write_string("Testing VFS file read/write...\n");

    let mut inode = InodeImpl::new("test.txt", FileType::RegularFile);
    inode.set_data(b"Test data".to_vec());
    let inode_ref: InodeRef = Arc::new(Mutex::new(inode));

    let mut file = crate::fs::vfs::file::File::new(inode_ref.clone(), FileFlags::read_write());

    let write_result = file.write(b"Hello").unwrap();
    let _ = file.seek(SeekFrom::Start(0));

    let mut buf = [0u8; 64];
    let read_result = file.read(&mut buf);

    let ok = write_result == 5 && read_result.is_ok() && &buf[..5] == b"Hello";

    serial::write_string(if ok {
        "test_vfs_file_read_write: OK\n"
    } else {
        "test_vfs_file_read_write: FAIL\n"
    });
}

pub fn test_vfs_mount_namespace() {
    serial::write_string("Testing VFS mount namespace...\n");

    let root: InodeRef = Arc::new(Mutex::new(InodeImpl::new("/", FileType::Directory)));
    let mut ns = MountNamespace::new(root.clone());

    let dev: InodeRef = Arc::new(Mutex::new(InodeImpl::new("dev", FileType::Directory)));
    let result = ns.mount("/mnt", Some(dev.clone()), root.clone());

    let ok = result.is_ok() && ns.find_mount("/mnt").is_some();

    serial::write_string(if ok {
        "test_vfs_mount_namespace: OK\n"
    } else {
        "test_vfs_mount_namespace: FAIL\n"
    });
}

pub fn test_vfs_dentry() {
    serial::write_string("Testing VFS dentry...\n");

    let child_inode: InodeRef =
        Arc::new(Mutex::new(InodeImpl::new("child", FileType::RegularFile)));

    let dentry = Arc::new(Mutex::new(Dentry::new("child", Some(child_inode), None)));

    let ok = dentry.lock().d_is_positive() && !dentry.lock().d_is_negative();

    serial::write_string(if ok {
        "test_vfs_dentry: OK\n"
    } else {
        "test_vfs_dentry: FAIL\n"
    });
}

pub fn test_vfs_dentry_lookup() {
    serial::write_string("Testing VFS dentry lookup...\n");

    let inode: InodeRef = Arc::new(Mutex::new(InodeImpl::new("test", FileType::RegularFile)));

    let dentry = Arc::new(Mutex::new(Dentry::new("test", Some(inode), None)));

    let lookup_result = dentry.lock().lookup("nonexistent");

    let ok = lookup_result.is_none();

    serial::write_string(if ok {
        "test_vfs_dentry_lookup: OK\n"
    } else {
        "test_vfs_dentry_lookup: FAIL\n"
    });
}

pub fn test_vfs_super_block() {
    serial::write_string("Testing VFS super block...\n");

    let root: InodeRef = Arc::new(Mutex::new(InodeImpl::new("/", FileType::Directory)));
    let sb = SuperBlock::new(1, root.clone(), "testfs");

    let ok = sb.s_dev == 1 && sb.s_fs_info == "testfs";

    serial::write_string(if ok {
        "test_vfs_super_block: OK\n"
    } else {
        "test_vfs_super_block: FAIL\n"
    });
}

pub fn test_vfs_symlink() {
    serial::write_string("Testing VFS symlink...\n");

    let mut dir = InodeImpl::new("parent", FileType::Directory);
    let result = dir.symlink("target_file", "link_name");

    let ok = result.is_ok() && dir.lookup("link_name").is_some();

    serial::write_string(if ok {
        "test_vfs_symlink: OK\n"
    } else {
        "test_vfs_symlink: FAIL\n"
    });
}

pub fn test_vfs_readlink() {
    serial::write_string("Testing VFS readlink...\n");

    let mut inode = InodeImpl::new("link", FileType::Symlink);
    inode.set_data(b"target_path".to_vec());
    let inode_ref: InodeRef = Arc::new(Mutex::new(inode));

    let link_content = inode_ref.lock().readlink();

    let ok = link_content.is_some() && link_content.unwrap() == "target_path";

    serial::write_string(if ok {
        "test_vfs_readlink: OK\n"
    } else {
        "test_vfs_readlink: FAIL\n"
    });
}

pub fn test_vfs_file_seek() {
    serial::write_string("Testing VFS file seek...\n");

    let mut inode = InodeImpl::new("test.txt", FileType::RegularFile);
    inode.set_data(b"0123456789".to_vec());
    let inode_ref: InodeRef = Arc::new(Mutex::new(inode));

    let mut file = crate::fs::vfs::file::File::new(inode_ref, FileFlags::read_write());

    let _ = file.seek(SeekFrom::Start(5));
    let pos = file.offset;

    let ok = pos == 5;

    serial::write_string(if ok {
        "test_vfs_file_seek: OK\n"
    } else {
        "test_vfs_file_seek: FAIL\n"
    });
}

pub fn test_vfs_file_truncate() {
    serial::write_string("Testing VFS file truncate...\n");

    let mut inode = InodeImpl::new("test.txt", FileType::RegularFile);
    inode.set_data(b"0123456789".to_vec());
    let inode_ref: InodeRef = Arc::new(Mutex::new(inode));

    let mut file = crate::fs::vfs::file::File::new(inode_ref, FileFlags::read_write());
    let _ = file.seek(SeekFrom::Start(3));
    let _ = file.truncate();

    let ok = file.stat().st_size == 3;

    serial::write_string(if ok {
        "test_vfs_file_truncate: OK\n"
    } else {
        "test_vfs_file_truncate: FAIL\n"
    });
}

pub fn test_vfs_file_dup() {
    serial::write_string("Testing VFS file dup...\n");

    let inode: InodeRef = Arc::new(Mutex::new(InodeImpl::new("test", FileType::RegularFile)));
    let mut table = FileTable::new();

    let file = crate::fs::vfs::file::File::new(inode, FileFlags::read_write());
    let fd1 = table.allocate_fd(Arc::new(Mutex::new(file)));
    let fd2 = table.dup(fd1.unwrap());

    let ok = fd2.is_some() && fd2 != fd1;

    serial::write_string(if ok {
        "test_vfs_file_dup: OK\n"
    } else {
        "test_vfs_file_dup: FAIL\n"
    });
}

pub fn test_vfs_readdir() {
    serial::write_string("Testing VFS readdir...\n");

    let mut dir = InodeImpl::new("test_dir", FileType::Directory);
    let _ = dir.mkdir("subdir1", FilePermissions::default_directory());
    let _ = dir.create("file1.txt", FilePermissions::default_file());

    let dir_ref: InodeRef = Arc::new(Mutex::new(dir));

    let entry0 = dir_ref.lock().readdir(0);
    let entry1 = dir_ref.lock().readdir(1);
    let entry2 = dir_ref.lock().readdir(2);

    let ok = entry0.is_some() && entry1.is_some() && entry2.is_none();

    serial::write_string(if ok {
        "test_vfs_readdir: OK\n"
    } else {
        "test_vfs_readdir: FAIL\n"
    });
}

pub fn test_vfs_mount_unmount() {
    serial::write_string("Testing VFS mount/unmount...\n");

    let root: InodeRef = Arc::new(Mutex::new(InodeImpl::new("/", FileType::Directory)));
    let mut ns = MountNamespace::new(root.clone());

    let dev: InodeRef = Arc::new(Mutex::new(InodeImpl::new("dev", FileType::Directory)));
    let _ = ns.mount("/mnt", Some(dev.clone()), root);

    let unmount_result = ns.unmount("/mnt");

    let ok = unmount_result.is_ok() && ns.find_mount("/mnt").is_none();

    serial::write_string(if ok {
        "test_vfs_mount_unmount: OK\n"
    } else {
        "test_vfs_mount_unmount: FAIL\n"
    });
}

pub fn test_vfs_walk_path() {
    serial::write_string("Testing VFS walk_path...\n");

    let root: InodeRef = Arc::new(Mutex::new(InodeImpl::new("/", FileType::Directory)));
    let mut root_guard = root.lock();
    let _ = root_guard.mkdir("home", FilePermissions::default_directory());
    let home = root_guard.lookup("home").unwrap();
    drop(root_guard);

    let mut home_guard = home.lock();
    let _ = home_guard.mkdir("user", FilePermissions::default_directory());
    drop(home_guard);

    let ns = MountNamespace::new(root);
    let result = ns.walk_path("/home/user");

    let ok = result.is_ok();

    serial::write_string(if ok {
        "test_vfs_walk_path: OK\n"
    } else {
        "test_vfs_walk_path: FAIL\n"
    });
}

pub fn test_initramfs_root_creation() {
    serial::write_string("Testing initramfs root creation...\n");

    let root = initramfs::create_initramfs_root();
    let root_lock = root.lock();

    let ok = root_lock.name() == "/"
        && root_lock.inode_type() == FileType::Directory
        && root_lock.stat().st_ino > 0;

    serial::write_string(if ok {
        "test_initramfs_root_creation: OK\n"
    } else {
        "test_initramfs_root_creation: FAIL\n"
    });
}

pub fn test_initramfs_children() {
    serial::write_string("Testing initramfs children...\n");

    let root = initramfs::create_initramfs_root();
    let root_lock = root.lock();

    let init_exists = root_lock.lookup("init").is_some();
    let hello_exists = root_lock.lookup("hello.txt").is_some();
    let bin_exists = root_lock.lookup("bin").is_some();
    let etc_exists = root_lock.lookup("etc").is_some();

    let ok = init_exists && hello_exists && bin_exists && etc_exists;

    serial::write_string(if ok {
        "test_initramfs_children: OK\n"
    } else {
        "test_initramfs_children: FAIL\n"
    });
}

pub fn test_initramfs_file_read() {
    serial::write_string("Testing initramfs file read...\n");

    let root = initramfs::create_initramfs_root();
    let root_lock = root.lock();
    let hello = root_lock.lookup("hello.txt").unwrap();
    drop(root_lock);

    let hello_lock = hello.lock();
    let mut buf = [0u8; 64];
    let n = hello_lock.read(0, &mut buf).unwrap();
    let content = core::str::from_utf8(&buf[..n]).unwrap();
    let ok = content.contains("Hello from Rust Kernel");

    serial::write_string(if ok {
        "test_initramfs_file_read: OK\n"
    } else {
        "test_initramfs_file_read: FAIL\n"
    });
}

pub fn test_initramfs_readdir() {
    serial::write_string("Testing initramfs readdir...\n");

    let root = initramfs::create_initramfs_root();
    let root_lock = root.lock();

    let entry0 = root_lock.readdir(0);
    let entry1 = root_lock.readdir(1);
    let entry2 = root_lock.readdir(2);
    let entry3 = root_lock.readdir(3);
    let entry4 = root_lock.readdir(4);
    let entry5 = root_lock.readdir(5);

    let ok = entry0.is_some()
        && entry1.is_some()
        && entry2.is_some()
        && entry3.is_some()
        && entry4.is_some()
        && entry5.is_none();

    serial::write_string(if ok {
        "test_initramfs_readdir: OK\n"
    } else {
        "test_initramfs_readdir: FAIL\n"
    });
}

pub fn test_tmpfs_root_creation() {
    serial::write_string("Testing tmpfs root creation...\n");

    let root = tmpfs::create_tmpfs_root();
    let root_lock = root.lock();

    let ok = root_lock.name() == "/"
        && root_lock.inode_type() == FileType::Directory
        && root_lock.stat().st_ino > 0;

    serial::write_string(if ok {
        "test_tmpfs_root_creation: OK\n"
    } else {
        "test_tmpfs_root_creation: FAIL\n"
    });
}

pub fn test_tmpfs_children() {
    serial::write_string("Testing tmpfs children...\n");

    let root = tmpfs::create_tmpfs_root();
    let root_lock = root.lock();

    let tmp_exists = root_lock.lookup("tmp").is_some();
    let var_exists = root_lock.lookup("var").is_some();
    let dev_exists = root_lock.lookup("dev").is_some();
    let proc_exists = root_lock.lookup("proc").is_some();

    let ok = tmp_exists && var_exists && dev_exists && proc_exists;

    serial::write_string(if ok {
        "test_tmpfs_children: OK\n"
    } else {
        "test_tmpfs_children: FAIL\n"
    });
}

pub fn test_tmpfs_mkdir() {
    serial::write_string("Testing tmpfs mkdir...\n");

    let root = tmpfs::create_tmpfs_root();
    let mut root_lock = root.lock();

    let result = root_lock.mkdir("testdir", FilePermissions::default_directory());
    let ok = result.is_some() && root_lock.lookup("testdir").is_some();

    serial::write_string(if ok {
        "test_tmpfs_mkdir: OK\n"
    } else {
        "test_tmpfs_mkdir: FAIL\n"
    });
}

pub fn test_tmpfs_create_file() {
    serial::write_string("Testing tmpfs create file...\n");

    let root = tmpfs::create_tmpfs_root();
    let mut root_lock = root.lock();

    let result = root_lock.create("testfile.txt", FilePermissions::default_file());
    let ok = result.is_some() && root_lock.lookup("testfile.txt").is_some();

    serial::write_string(if ok {
        "test_tmpfs_create_file: OK\n"
    } else {
        "test_tmpfs_create_file: FAIL\n"
    });
}

pub fn test_tmpfs_read_write() {
    serial::write_string("Testing tmpfs read/write...\n");

    let root = tmpfs::create_tmpfs_root();
    let mut root_lock = root.lock();

    let file = root_lock.create("test.txt", FilePermissions::default_file());
    let file = file.unwrap();
    drop(root_lock);

    let mut file_lock = file.lock();
    let write_result = file_lock.write(0, b"Hello, tmpfs!");
    drop(file_lock);

    let read_result = file.lock().read(0, &mut [0u8; 64]);
    let ok = write_result.is_ok() && read_result.is_ok();

    serial::write_string(if ok {
        "test_tmpfs_read_write: OK\n"
    } else {
        "test_tmpfs_read_write: FAIL\n"
    });
}

pub fn test_tmpfs_unlink() {
    serial::write_string("Testing tmpfs unlink...\n");

    let root = tmpfs::create_tmpfs_root();
    let mut root_lock = root.lock();

    let _ = root_lock.create("file.txt", FilePermissions::default_file());
    let result = root_lock.unlink("file.txt");
    let ok = result.is_ok() && root_lock.lookup("file.txt").is_none();

    serial::write_string(if ok {
        "test_tmpfs_unlink: OK\n"
    } else {
        "test_tmpfs_unlink: FAIL\n"
    });
}

pub fn test_tmpfs_rmdir() {
    serial::write_string("Testing tmpfs rmdir...\n");

    let root = tmpfs::create_tmpfs_root();
    let mut root_lock = root.lock();

    let _ = root_lock.mkdir("subdir", FilePermissions::default_directory());
    let result = root_lock.rmdir("subdir");
    let ok = result.is_ok() && root_lock.lookup("subdir").is_none();

    serial::write_string(if ok {
        "test_tmpfs_rmdir: OK\n"
    } else {
        "test_tmpfs_rmdir: FAIL\n"
    });
}

pub fn test_tmpfs_readdir() {
    serial::write_string("Testing tmpfs readdir...\n");

    let root = tmpfs::create_tmpfs_root();
    let root_lock = root.lock();

    let entry0 = root_lock.readdir(0);
    let entry1 = root_lock.readdir(1);
    let entry2 = root_lock.readdir(2);
    let entry3 = root_lock.readdir(3);
    let entry4 = root_lock.readdir(4);

    let ok = entry0.is_some()
        && entry1.is_some()
        && entry2.is_some()
        && entry3.is_some()
        && entry4.is_none();

    serial::write_string(if ok {
        "test_tmpfs_readdir: OK\n"
    } else {
        "test_tmpfs_readdir: FAIL\n"
    });
}

pub fn test_tmpfs_truncate() {
    serial::write_string("Testing tmpfs truncate...\n");

    let root = tmpfs::create_tmpfs_root();
    let mut root_lock = root.lock();

    let file = root_lock
        .create("test.txt", FilePermissions::default_file())
        .unwrap();
    drop(root_lock);

    {
        let mut file_lock = file.lock();
        let _ = file_lock.write(0, b"0123456789");
    }

    let mut file_lock = file.lock();
    let result = file_lock.truncate(5);
    let size = file_lock.stat().st_size;
    drop(file_lock);

    let ok = result.is_ok() && size == 5;

    serial::write_string(if ok {
        "test_tmpfs_truncate: OK\n"
    } else {
        "test_tmpfs_truncate: FAIL\n"
    });
}

pub fn test_tmpfs_symlink() {
    serial::write_string("Testing tmpfs symlink...\n");

    let root = tmpfs::create_tmpfs_root();
    let mut root_lock = root.lock();

    let result = root_lock.symlink("target_file", "link_name");
    let ok = result.is_ok() && root_lock.lookup("link_name").is_some();

    serial::write_string(if ok {
        "test_tmpfs_symlink: OK\n"
    } else {
        "test_tmpfs_symlink: FAIL\n"
    });
}

pub fn test_tmpfs_readlink() {
    serial::write_string("Testing tmpfs readlink...\n");

    let root = tmpfs::create_tmpfs_root();
    let mut root_lock = root.lock();

    let _ = root_lock.symlink("target_path", "link_name");
    drop(root_lock);

    let link = root.lock().lookup("link_name").unwrap();
    let link_content = link.lock().readlink();

    let ok = link_content.is_some() && link_content.unwrap() == "target_path";

    serial::write_string(if ok {
        "test_tmpfs_readlink: OK\n"
    } else {
        "test_tmpfs_readlink: FAIL\n"
    });
}
