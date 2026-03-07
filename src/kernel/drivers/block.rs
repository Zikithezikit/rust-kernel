//! Block device interface (Linux-inspired)
//!
//! Provides traits and structures for block-oriented storage devices.
//! Inspired by Linux's block layer (block/blk-core.c, include/linux/blkdev.h)

use crate::include::error::KernelResult;

/// Standard sector size for ATA devices
pub const ATA_SECTOR_SIZE: usize = 512;

/// Standard block size (may differ from sector size)
pub const BLOCK_SIZE: usize = 4096;

/// Maximum sectors per request (Linux uses 256 max)
pub const ATA_MAX_SECTORS: usize = 256;

/// ATA register offsets (Linux: include/linux/ata.h)
pub mod ata_regs {
    pub const ATA_REG_DATA: u16 = 0x00;
    pub const ATA_REG_ERR: u16 = 0x01;
    pub const ATA_REG_NSECT: u16 = 0x02;
    pub const ATA_REG_LBAL: u16 = 0x03;
    pub const ATA_REG_LBAM: u16 = 0x04;
    pub const ATA_REG_LBAH: u16 = 0x05;
    pub const ATA_REG_DEVICE: u16 = 0x06;
    pub const ATA_REG_STATUS: u16 = 0x07;

    pub const ATA_REG_FEATURE: u16 = ATA_REG_ERR;
    pub const ATA_REG_CMD: u16 = ATA_REG_STATUS;
}

/// ATA status bits (Linux: include/linux/ata.h)
pub mod ata_status {
    pub const ATA_BUSY: u8 = 0x80;
    pub const ATA_DRDY: u8 = 0x40;
    pub const ATA_DF: u8 = 0x20;
    pub const ATA_SRV: u8 = 0x10;
    pub const ATA_DRQ: u8 = 0x08;
    pub const ATA_CORR: u8 = 0x04;
    pub const ATA_SENSE: u8 = 0x02;
    pub const ATA_ERR: u8 = 0x01;
}

/// ATA error bits
pub mod ata_error {
    pub const ATA_AMNF: u8 = 0x01;
    pub const ATA_TK0NF: u8 = 0x02;
    pub const ATA_ABRT: u8 = 0x04;
    pub const ATA_MCR: u8 = 0x08;
    pub const ATA_IDNF: u8 = 0x10;
    pub const ATA_MC: u8 = 0x20;
    pub const ATA_UNC: u8 = 0x40;
    pub const ATA_BBK: u8 = 0x80;
}

/// ATA commands
pub mod ata_cmd {
    pub const ATA_CMD_READ_PIO: u8 = 0x20;
    pub const ATA_CMD_READ_PIO_EXT: u8 = 0x24;
    pub const ATA_CMD_READ_DMA: u8 = 0xC8;
    pub const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
    pub const ATA_CMD_WRITE_PIO: u8 = 0x30;
    pub const ATA_CMD_WRITE_PIO_EXT: u8 = 0x34;
    pub const ATA_CMD_WRITE_DMA: u8 = 0xCA;
    pub const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
    pub const ATA_CMD_IDENTIFY: u8 = 0xEC;
    pub const ATA_CMD_SET_FEATURES: u8 = 0xEF;
    pub const ATA_CMD_FLUSH_CACHE: u8 = 0xE7;
    pub const ATA_CMD_FLUSH_CACHE_EXT: u8 = 0xEA;
}

/// ATA device/head register bits
pub mod ata_device {
    pub const ATA_LBA: u8 = 0x40;
    pub const ATA_DEV1: u8 = 0x10;
    pub const ATA_HOB: u8 = 0x80;

    pub const ATA_MASTER_MAGIC: u8 = 0xE0;
    pub const ATA_SLAVE_MAGIC: u8 = 0xF0;
}

/// I/O port base addresses
pub mod ata_port {
    pub const ATA_PRIMARY: u16 = 0x1F0;
    pub const ATA_PRIMARY_CTRL: u16 = 0x3F6;
    pub const ATA_SECONDARY: u16 = 0x170;
    pub const ATA_SECONDARY_CTRL: u16 = 0x376;
}

pub mod ata_ctrl {
    pub const SRST: u8 = 0x04;
    pub const NIEN: u8 = 0x02;
    pub const ENABLE: u8 = 0x00;
}

/// ATA limits and boundaries
pub mod ata_limits {
    pub const LBA28_MAX_SECTORS: u64 = 0x10000000;
    pub const LBA48_MAX_SECTORS: u64 = 0x1000000000;
}

/// ATA timeout settings
pub mod ata_timeouts {
    pub const BUSY_TIMEOUT: u32 = 100_000;
    pub const DRQ_TIMEOUT: u32 = 30_000;
    pub const RESET_DELAY: u32 = 1_000;
}

/// ATA IDENTIFY data field indices
pub mod ata_identify {
    pub const IDENTIFY_WORDS: usize = 256;
    pub const WORDS_PER_SECTOR: usize = 256;

    pub const SERIAL_START: usize = 10;
    pub const SERIAL_LEN: usize = 10;

    pub const MODEL_START: usize = 27;
    pub const MODEL_LEN: usize = 20;

    pub const LBA28_CAPACITY_START: usize = 60;
    pub const LBA48_SUPPORT_BIT_WORD: usize = 83;
    pub const LBA48_SUPPORT_BIT_MASK: u16 = 0x0400;
    pub const LBA48_CAPACITY_START: usize = 100;
}

/// Block device trait (Linux: block_device operations)
///
/// This trait provides the interface for block devices like ATA disks.
/// In Linux, this corresponds to the block_device_operations structure.
pub trait BlockDevice: Send + Sync {
    /// Read one or more sectors from the device
    ///
    /// # Arguments
    /// * `sector` - Starting LBA sector number
    /// * `count` - Number of sectors to read
    /// * `buf` - Buffer to store data (must be at least count * sector_size)
    ///
    /// # Returns
    /// Number of bytes read, or error
    fn read_sectors(&self, sector: u64, count: usize, buf: &mut [u8]) -> KernelResult<usize>;

    /// Write one or more sectors to the device
    ///
    /// # Arguments
    /// * `sector` - Starting LBA sector number
    /// * `count` - Number of sectors to write
    /// * `buf` - Data to write (must be at least count * sector_size)
    ///
    /// # Returns
    /// Number of bytes written, or error
    fn write_sectors(&self, sector: u64, count: usize, buf: &[u8]) -> KernelResult<usize>;

    /// Get total number of sectors
    fn num_sectors(&self) -> u64;

    /// Get sector size (typically 512 bytes)
    fn sector_size(&self) -> usize;

    /// Check if device is present
    fn is_present(&self) -> bool;

    /// Get device name/identifier
    fn device_name(&self) -> &str;
}

/// Block device request structure
/// Linux equivalent: struct request
#[derive(Debug)]
pub struct BlockRequest {
    pub sector: u64,
    pub count: usize,
    pub buffer: *mut u8,
    pub is_write: bool,
}

/// Block device statistics
/// Linux equivalent: struct hd_struct
#[derive(Debug, Default)]
pub struct BlockStats {
    pub reads: u64,
    pub writes: u64,
    pub read_sectors: u64,
    pub write_sectors: u64,
    pub read_errors: u64,
    pub write_errors: u64,
}
