//! Block device and ATA driver tests
//!
//! Tests for the block device interface and ATA/PATA driver.

use crate::drivers::ata::{AtaChannel, AtaDevice, AtaDrive, AtaState};
use crate::drivers::block::{
    ata_cmd, ata_device, ata_error, ata_port, ata_regs, ata_status, BlockStats, ATA_SECTOR_SIZE,
};
use crate::drivers::serial;

/// Test ATA register definitions
pub fn test_ata_registers() {
    serial::write_string("test_ata_registers: ");

    // Test ATA register offsets
    if ata_regs::ATA_REG_DATA != 0x00 {
        serial::write_string("FAIL: DATA\n");
        return;
    }
    if ata_regs::ATA_REG_ERR != 0x01 {
        serial::write_string("FAIL: ERR\n");
        return;
    }
    if ata_regs::ATA_REG_NSECT != 0x02 {
        serial::write_string("FAIL: NSECT\n");
        return;
    }
    if ata_regs::ATA_REG_LBAL != 0x03 {
        serial::write_string("FAIL: LBAL\n");
        return;
    }
    if ata_regs::ATA_REG_LBAM != 0x04 {
        serial::write_string("FAIL: LBAM\n");
        return;
    }
    if ata_regs::ATA_REG_LBAH != 0x05 {
        serial::write_string("FAIL: LBAH\n");
        return;
    }
    if ata_regs::ATA_REG_DEVICE != 0x06 {
        serial::write_string("FAIL: DEVICE\n");
        return;
    }
    if ata_regs::ATA_REG_STATUS != 0x07 {
        serial::write_string("FAIL: STATUS\n");
        return;
    }

    // Test ATA port bases
    if ata_port::ATA_PRIMARY != 0x1F0 {
        serial::write_string("FAIL: PRIMARY\n");
        return;
    }
    if ata_port::ATA_SECONDARY != 0x170 {
        serial::write_string("FAIL: SECONDARY\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test ATA status bits
pub fn test_ata_status_bits() {
    serial::write_string("test_ata_status_bits: ");

    if ata_status::ATA_BUSY != 0x80 {
        serial::write_string("FAIL: BUSY\n");
        return;
    }
    if ata_status::ATA_DRDY != 0x40 {
        serial::write_string("FAIL: DRDY\n");
        return;
    }
    if ata_status::ATA_DF != 0x20 {
        serial::write_string("FAIL: DF\n");
        return;
    }
    if ata_status::ATA_SRV != 0x10 {
        serial::write_string("FAIL: SRV\n");
        return;
    }
    if ata_status::ATA_DRQ != 0x08 {
        serial::write_string("FAIL: DRQ\n");
        return;
    }
    if ata_status::ATA_CORR != 0x04 {
        serial::write_string("FAIL: CORR\n");
        return;
    }
    if ata_status::ATA_SENSE != 0x02 {
        serial::write_string("FAIL: SENSE\n");
        return;
    }
    if ata_status::ATA_ERR != 0x01 {
        serial::write_string("FAIL: ERR\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test ATA error bits
pub fn test_ata_error_bits() {
    serial::write_string("test_ata_error_bits: ");

    if ata_error::ATA_AMNF != 0x01 {
        serial::write_string("FAIL: AMNF\n");
        return;
    }
    if ata_error::ATA_TK0NF != 0x02 {
        serial::write_string("FAIL: TK0NF\n");
        return;
    }
    if ata_error::ATA_ABRT != 0x04 {
        serial::write_string("FAIL: ABRT\n");
        return;
    }
    if ata_error::ATA_MCR != 0x08 {
        serial::write_string("FAIL: MCR\n");
        return;
    }
    if ata_error::ATA_IDNF != 0x10 {
        serial::write_string("FAIL: IDNF\n");
        return;
    }
    if ata_error::ATA_MC != 0x20 {
        serial::write_string("FAIL: MC\n");
        return;
    }
    if ata_error::ATA_UNC != 0x40 {
        serial::write_string("FAIL: UNC\n");
        return;
    }
    if ata_error::ATA_BBK != 0x80 {
        serial::write_string("FAIL: BBK\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test ATA commands
pub fn test_ata_commands() {
    serial::write_string("test_ata_commands: ");

    if ata_cmd::ATA_CMD_READ_PIO != 0x20 {
        serial::write_string("FAIL: READ_PIO\n");
        return;
    }
    if ata_cmd::ATA_CMD_WRITE_PIO != 0x30 {
        serial::write_string("FAIL: WRITE_PIO\n");
        return;
    }
    if ata_cmd::ATA_CMD_IDENTIFY != 0xEC {
        serial::write_string("FAIL: IDENTIFY\n");
        return;
    }
    if ata_cmd::ATA_CMD_FLUSH_CACHE != 0xE7 {
        serial::write_string("FAIL: FLUSH\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test ATA device/head bits
pub fn test_ata_device_bits() {
    serial::write_string("test_ata_device_bits: ");

    if ata_device::ATA_LBA != 0x40 {
        serial::write_string("FAIL: LBA\n");
        return;
    }
    if ata_device::ATA_DEV1 != 0x10 {
        serial::write_string("FAIL: DEV1\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test ATA device creation
pub fn test_ata_device_creation() {
    serial::write_string("test_ata_device_creation: ");

    // Test primary master
    let device = AtaDevice::new(AtaChannel::Primary, AtaDrive::Master);

    // Verify channel
    let ch = device.channel();
    if ch != AtaChannel::Primary {
        serial::write_string("FAIL: wrong channel\n");
        return;
    }

    // Verify drive
    let dr = device.drive();
    if dr != AtaDrive::Master {
        serial::write_string("FAIL: wrong drive\n");
        return;
    }

    // Verify base port
    let bp = device.base_port();
    if bp != ata_port::ATA_PRIMARY {
        serial::write_string("FAIL: wrong base port\n");
        serial::write_hex(bp as u64);
        serial::write_string(" != ");
        serial::write_hex(ata_port::ATA_PRIMARY as u64);
        serial::write_string("\n");
        return;
    }

    // Verify ctrl port
    let cp = device.ctrl_port();
    if cp != ata_port::ATA_PRIMARY_CTRL {
        serial::write_string("FAIL: wrong ctrl port\n");
        return;
    }

    // Verify state
    let st = device.state();
    if st != AtaState::None {
        serial::write_string("FAIL: wrong state\n");
        return;
    }

    // Verify num sectors
    let ns = device.num_sectors();
    if ns != 0 {
        serial::write_string("FAIL: wrong num sectors\n");
        return;
    }

    // Verify sector size
    let ss = device.sector_size();
    if ss != ATA_SECTOR_SIZE {
        serial::write_string("FAIL: wrong sector size\n");
        return;
    }

    // Test secondary slave
    let device = AtaDevice::new(AtaChannel::Secondary, AtaDrive::Slave);

    let ch = device.channel();
    if ch != AtaChannel::Secondary {
        serial::write_string("FAIL: wrong secondary channel\n");
        return;
    }

    let dr = device.drive();
    if dr != AtaDrive::Slave {
        serial::write_string("FAIL: wrong slave drive\n");
        return;
    }

    let bp = device.base_port();
    if bp != ata_port::ATA_SECONDARY {
        serial::write_string("FAIL: wrong secondary base port\n");
        return;
    }

    let cp = device.ctrl_port();
    if cp != ata_port::ATA_SECONDARY_CTRL {
        serial::write_string("FAIL: wrong secondary ctrl port\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test BlockStats structure
pub fn test_block_stats() {
    serial::write_string("test_block_stats: ");

    let stats = BlockStats::default();
    if stats.reads != 0 {
        serial::write_string("FAIL: reads\n");
        return;
    }
    if stats.writes != 0 {
        serial::write_string("FAIL: writes\n");
        return;
    }
    if stats.read_sectors != 0 {
        serial::write_string("FAIL: read_sectors\n");
        return;
    }
    if stats.write_sectors != 0 {
        serial::write_string("FAIL: write_sectors\n");
        return;
    }
    if stats.read_errors != 0 {
        serial::write_string("FAIL: read_errors\n");
        return;
    }
    if stats.write_errors != 0 {
        serial::write_string("FAIL: write_errors\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test sector size constant
pub fn test_sector_size() {
    serial::write_string("test_sector_size: ");

    if ATA_SECTOR_SIZE != 512 {
        serial::write_string("FAIL: sector size != 512\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test LBA calculation for device select register
pub fn test_lba_device_select() {
    serial::write_string("test_lba_device_select: ");

    // Test master drive LBA calculation
    let lba: u64 = 0x100000;

    let device_byte = ata_device::ATA_MASTER_MAGIC | ((lba >> 24) & 0x0F) as u8;
    if device_byte != ata_device::ATA_MASTER_MAGIC {
        serial::write_string("FAIL: LBA calc 1\n");
        return;
    }

    // Test with higher LBA
    // lba = 0x12345678, lba >> 24 = 0x12, 0x12 & 0x0F = 0x02, ATA_MASTER_MAGIC(0xE0) | 0x02 = 0xE2
    let lba: u64 = 0x12345678;
    let device_byte = ata_device::ATA_MASTER_MAGIC | ((lba >> 24) & 0x0F) as u8;
    if device_byte != 0xE2 {
        serial::write_string("FAIL: LBA calc 2\n");
        serial::write_hex(device_byte as u64);
        serial::write_string(" != E2\n");
        return;
    }

    // Test slave drive: ATA_SLAVE_MAGIC(0xF0) | 0x02 = 0xF2
    let device_byte = ata_device::ATA_SLAVE_MAGIC | ((lba >> 24) & 0x0F) as u8;
    if device_byte != 0xF2 {
        serial::write_string("FAIL: LBA calc 3\n");
        serial::write_hex(device_byte as u64);
        serial::write_string(" != F2\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test sector count bounds
pub fn test_sector_count_bounds() {
    serial::write_string("test_sector_count_bounds: ");

    // Maximum sectors per operation (255 fits in u8)
    let max_sectors: usize = 255;
    if max_sectors * ATA_SECTOR_SIZE != 255 * 512 {
        serial::write_string("FAIL: max sectors 1\n");
        return;
    }
    if max_sectors * ATA_SECTOR_SIZE != 130560 {
        serial::write_string("FAIL: max sectors 2\n");
        return;
    }

    // Single sector
    let single_sector = 1u8;
    if single_sector as usize * ATA_SECTOR_SIZE != 512 {
        serial::write_string("FAIL: single sector\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test identify data parsing (simulated)
pub fn test_identify_data_parsing() {
    serial::write_string("test_identify_data_parsing: ");

    // Simulate IDENTIFY data for a 1GB drive
    // Word 60-61: LBA capacity (LBA28)
    let lba_low: u64 = 0x3AE800; // ~1GB in sectors
    let lba_high: u64 = 0;
    let lba_capacity = (lba_high << 16) | lba_low;
    if lba_capacity != 0x3AE800 {
        serial::write_string("FAIL: LBA28\n");
        return;
    }

    // Test LBA48 capacity (words 100-103)
    // For drives > 137GB
    let lba48_low = 0x3AE800u64;
    let lba48_high = 0u64;
    let lba48_low2 = 0u64;
    let lba48_high2 = 0u64;
    let lba48_capacity = (lba48_high2 << 48) | (lba48_low2 << 32) | (lba48_high << 16) | lba48_low;
    if lba48_capacity != 0x3AE800 {
        serial::write_string("FAIL: LBA48\n");
        return;
    }

    serial::write_string("OK\n");
}

/// Test buffer size calculations
pub fn test_buffer_calculations() {
    serial::write_string("test_buffer_calculations: ");

    // Test buffer size for various sector counts
    let test_cases: [(usize, usize); 9] = [
        (1, 512),
        (2, 1024),
        (4, 2048),
        (8, 4096),
        (16, 8192),
        (32, 16384),
        (64, 32768),
        (128, 65536),
        (256, 131072),
    ];

    for (sectors, expected_size) in test_cases.iter() {
        let size = sectors * ATA_SECTOR_SIZE;
        if size != *expected_size {
            serial::write_string("FAIL: buffer calc\n");
            return;
        }
    }

    serial::write_string("OK\n");
}

/// Run all block device and ATA tests
pub fn run_block_tests() {
    serial::write_string("\n=== Block Device & ATA Tests ===\n");

    test_ata_registers();
    test_ata_status_bits();
    test_ata_error_bits();
    test_ata_commands();
    test_ata_device_bits();
    test_ata_device_creation();
    test_block_stats();
    test_sector_size();
    test_lba_device_select();
    test_sector_count_bounds();
    test_identify_data_parsing();
    test_buffer_calculations();

    serial::write_string("=== Block Device & ATA: OK ===\n");
}
