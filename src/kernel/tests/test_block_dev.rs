use crate::drivers::serial;
use crate::fs::vfs::inode::FileType;
use crate::kernel_instance::kernel;

pub fn test_block_dev_mounting() {
    serial::write_string("Testing Block Device mounting at /dev/hda...\n");

    let ok = if let Some(ns) = kernel().mount_ns() {
        if let Ok(hda) = ns.walk_path("/dev/hda") {
            let hda_lock = hda.lock();
            hda_lock.inode_type() == FileType::BlockDevice && hda_lock.name() == "hda"
        } else {
            false
        }
    } else {
        false
    };

    serial::write_string(if ok {
        "test_block_dev_mounting: OK\n"
    } else {
        "test_block_dev_mounting: FAIL\n"
    });
}

pub fn test_block_dev_read() {
    serial::write_string("Testing Block Device read from /dev/hda...\n");

    let ok = if let Some(ns) = kernel().mount_ns() {
        if let Ok(hda) = ns.walk_path("/dev/hda") {
            let hda_lock = hda.lock();
            let mut buf = [0u8; 512];
            // Read first sector
            match hda_lock.read(0, &mut buf) {
                Ok(n) => n == 512,
                Err(_) => false,
            }
        } else {
            false
        }
    } else {
        false
    };

    serial::write_string(if ok {
        "test_block_dev_read: OK\n"
    } else {
        "test_block_dev_read: FAIL\n"
    });
}
