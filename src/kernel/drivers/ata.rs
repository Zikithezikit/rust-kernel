//! ATA/PATA Disk Driver
//!
//! Implements support for ATA/IDE hard drives using PIO (Programmed I/O) mode.
//! Inspired by Linux's libata and legacy IDE drivers.

use crate::drivers::block::{
    ata_cmd, ata_ctrl, ata_device, ata_identify, ata_limits, ata_port, ata_regs, ata_status,
    ata_timeouts, BlockDevice, BlockStats, ATA_SECTOR_SIZE,
};
use crate::drivers::serial;
use crate::include::error::{KernelError, KernelResult};
use alloc::sync::Arc;
use spin::Mutex;
use x86_64::instructions::port::Port;

/// ATA channel (primary/secondary)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtaChannel {
    Primary,
    Secondary,
}

/// ATA drive (master/slave)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtaDrive {
    Master,
    Slave,
}

/// ATA device state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtaState {
    None,
    Present,
    Identified,
    Error,
}

/// ATA device (Linux: ata_device)
///
/// Represents a single ATA disk on a channel.
pub struct AtaDevice {
    channel: AtaChannel,
    drive: AtaDrive,
    base_port: u16,
    ctrl_port: u16,
    state: AtaState,
    num_sectors: u64,
    sector_size: usize,
    stats: BlockStats,
    model: [u8; 40],
    serial: [u8; 20],
}

impl AtaDevice {
    /// Create a new ATA device
    pub fn new(channel: AtaChannel, drive: AtaDrive) -> Self {
        let (base_port, ctrl_port) = match channel {
            AtaChannel::Primary => (ata_port::ATA_PRIMARY, ata_port::ATA_PRIMARY_CTRL),
            AtaChannel::Secondary => (ata_port::ATA_SECONDARY, ata_port::ATA_SECONDARY_CTRL),
        };

        Self {
            channel,
            drive,
            base_port,
            ctrl_port,
            state: AtaState::None,
            num_sectors: 0,
            sector_size: ATA_SECTOR_SIZE,
            stats: BlockStats::default(),
            model: [0; 40],
            serial: [0; 20],
        }
    }

    /// Get the ATA channel
    pub fn channel(&self) -> AtaChannel {
        self.channel
    }

    /// Get the ATA drive (master/slave)
    pub fn drive(&self) -> AtaDrive {
        self.drive
    }

    /// Get the base I/O port
    pub fn base_port(&self) -> u16 {
        self.base_port
    }

    /// Get the control port
    pub fn ctrl_port(&self) -> u16 {
        self.ctrl_port
    }

    /// Get the device state
    pub fn state(&self) -> AtaState {
        self.state
    }

    /// Get the number of sectors
    pub fn num_sectors(&self) -> u64 {
        self.num_sectors
    }

    /// Get the sector size
    pub fn sector_size(&self) -> usize {
        self.sector_size
    }

    /// Disable interrupts for this device
    pub fn disable_interrupts(&self) {
        let mut port: Port<u8> = Port::new(self.ctrl_port);
        unsafe { port.write(ata_ctrl::NIEN) };
    }

    /// Wait for device to not be busy (Linux: ata_wait_idle)
    fn wait_not_busy(&self) -> KernelResult<()> {
        let mut port: Port<u8> = Port::new(self.base_port + ata_regs::ATA_REG_STATUS);
        let mut timeout = 0u32;

        loop {
            let status = unsafe { port.read() };
            if status & ata_status::ATA_BUSY == 0 {
                return Ok(());
            }
            timeout += 1;
            if timeout > ata_timeouts::BUSY_TIMEOUT {
                return Err(KernelError::DeviceError("ATA busy timeout"));
            }
        }
    }

    /// Wait for DRQ (data request)
    fn wait_drq(&self, timeout: u32) -> KernelResult<()> {
        let mut port: Port<u8> = Port::new(self.base_port + ata_regs::ATA_REG_STATUS);
        let mut timeout_counter = 0u32;

        loop {
            let status = unsafe { port.read() };
            if status & ata_status::ATA_DRQ != 0 {
                return Ok(());
            }
            if status & ata_status::ATA_ERR != 0 {
                return Err(KernelError::DeviceError("ATA DRQ error"));
            }
            timeout_counter += 1;
            if timeout_counter > timeout {
                return Err(KernelError::DeviceError("ATA DRQ timeout"));
            }
        }
    }

    /// Read a byte from an ATA register
    fn read_reg(&self, reg: u16) -> u8 {
        let mut port = Port::new(self.base_port + reg);
        unsafe { port.read() }
    }

    /// Write a byte to an ATA register
    fn write_reg(&self, reg: u16, value: u8) {
        let mut port = Port::new(self.base_port + reg);
        unsafe { port.write(value) };
    }

    /// Select device (master/slave)
    fn select_device(&self, is_lba: bool, lba_top: u8) {
        let mut device = match self.drive {
            AtaDrive::Master => ata_device::ATA_MASTER_MAGIC,
            AtaDrive::Slave => ata_device::ATA_SLAVE_MAGIC,
        };
        if is_lba {
            device |= ata_device::ATA_LBA;
        }
        device |= lba_top & 0x0F;
        self.write_reg(ata_regs::ATA_REG_DEVICE, device);
    }

    /// Soft reset (Linux: ata_do_soft_reset)
    fn soft_reset(&self) -> KernelResult<()> {
        let mut ctrl_port: Port<u8> = Port::new(self.ctrl_port);

        // Set SRST bit
        unsafe { ctrl_port.write(ata_ctrl::SRST) };

        // Wait a bit
        for _ in 0..ata_timeouts::RESET_DELAY {
            x86_64::instructions::nop();
        }

        // Clear SRST bit
        unsafe { ctrl_port.write(ata_ctrl::ENABLE) };

        // Wait for device to become ready
        self.wait_not_busy()
    }

    /// Identify device (Linux: ata_dev_identify)
    pub fn identify(&mut self) -> KernelResult<()> {
        // Disable interrupts first
        self.disable_interrupts();

        // Wait for not busy
        self.wait_not_busy()?;

        // Select device
        self.select_device(false, 0);

        // Send IDENTIFY command
        self.write_reg(ata_regs::ATA_REG_NSECT, 0);
        self.write_reg(ata_regs::ATA_REG_LBAL, 0);
        self.write_reg(ata_regs::ATA_REG_LBAM, 0);
        self.write_reg(ata_regs::ATA_REG_LBAH, 0);
        self.write_reg(ata_regs::ATA_REG_CMD, ata_cmd::ATA_CMD_IDENTIFY);

        // Check if device exists
        let status = self.read_reg(ata_regs::ATA_REG_STATUS);
        if status == 0 {
            return Err(KernelError::DeviceNotFound);
        }

        // Wait for not busy
        self.wait_not_busy()?;

        // Check for errors
        let status = self.read_reg(ata_regs::ATA_REG_STATUS);
        if status & ata_status::ATA_ERR != 0 {
            let error = self.read_reg(ata_regs::ATA_REG_ERR);
            serial::write_string("ATA: Identify error: ");
            serial::write_hex(error as u64);
            serial::write_string("\n");
            return Err(KernelError::DeviceError("ATA identify failed"));
        }

        // Wait for DRQ
        self.wait_drq(ata_timeouts::DRQ_TIMEOUT)?;

        // Read identify data (256 words = 512 bytes)
        let mut data_port = Port::new(self.base_port + ata_regs::ATA_REG_DATA);
        let mut identify_data = [0u16; ata_identify::IDENTIFY_WORDS];
        for i in 0..ata_identify::IDENTIFY_WORDS {
            identify_data[i] = unsafe { data_port.read() };
        }

        // Parse LBA capacity (words 60-61 for LBA28)
        let lba_low = identify_data[ata_identify::LBA28_CAPACITY_START] as u64;
        let lba_high = identify_data[ata_identify::LBA28_CAPACITY_START + 1] as u64;
        let lba_capacity = (lba_high << 16) | lba_low;

        // Validate LBA capacity - must be non-zero and reasonable
        if lba_capacity > 0 && lba_capacity < ata_limits::LBA28_MAX_SECTORS {
            self.num_sectors = lba_capacity;
        }

        // Check for LBA48 support (word 83, bit 10)
        if (identify_data[ata_identify::LBA48_SUPPORT_BIT_WORD]
            & ata_identify::LBA48_SUPPORT_BIT_MASK)
            != 0
        {
            let lba_low_48 = identify_data[ata_identify::LBA48_CAPACITY_START] as u64;
            let lba_high_48 = identify_data[ata_identify::LBA48_CAPACITY_START + 1] as u64;
            let lba_low2_48 = identify_data[ata_identify::LBA48_CAPACITY_START + 2] as u64;
            let lba_high2_48 = identify_data[ata_identify::LBA48_CAPACITY_START + 3] as u64;
            let lba48_capacity =
                (lba_high2_48 << 48) | (lba_low2_48 << 32) | (lba_high_48 << 16) | lba_low_48;
            // Use LBA48 only if it's reasonable
            if lba48_capacity > lba_capacity && lba48_capacity < ata_limits::LBA48_MAX_SECTORS {
                self.num_sectors = lba48_capacity;
            }
        }

        // Default to 512 bytes per sector - this is the standard
        // The sector size field in IDENTIFY is complex and often 0 for standard disks
        self.sector_size = ATA_SECTOR_SIZE;

        // Get model string (words 27-46)
        for i in 0..ata_identify::MODEL_LEN {
            let word = identify_data[ata_identify::MODEL_START + i];
            self.model[i * 2] = (word & 0xFF) as u8;
            self.model[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
        }

        // Get serial number (words 10-19)
        for i in 0..ata_identify::SERIAL_LEN {
            let word = identify_data[ata_identify::SERIAL_START + i];
            self.serial[i * 2] = (word & 0xFF) as u8;
            self.serial[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
        }

        self.state = AtaState::Identified;

        serial::write_string("ATA: ");
        match self.channel {
            AtaChannel::Primary => serial::write_string("Primary "),
            AtaChannel::Secondary => serial::write_string("Secondary "),
        }
        match self.drive {
            AtaDrive::Master => serial::write_string("Master"),
            AtaDrive::Slave => serial::write_string("Slave"),
        }
        serial::write_string(" - ");
        serial::write_hex(self.num_sectors);
        serial::write_string(" sectors, ");
        serial::write_hex(self.sector_size as u64);
        serial::write_string(" bytes/sector\n");

        Ok(())
    }

    /// Read sectors using PIO (Linux: ata_pio_read_sectors)
    fn read_sectors_pio(
        &mut self,
        start_lba: u64,
        count: usize,
        buf: &mut [u8],
    ) -> KernelResult<usize> {
        let expected_bytes = count * self.sector_size;
        if buf.len() < expected_bytes {
            return Err(KernelError::BufferTooSmall);
        }

        // Wait for device
        self.wait_not_busy()?;

        // Select device with LBA
        let device = match self.drive {
            AtaDrive::Master => ata_device::ATA_MASTER_MAGIC,
            AtaDrive::Slave => ata_device::ATA_SLAVE_MAGIC,
        } | ata_device::ATA_LBA
            | ((start_lba >> 24) & 0x0F) as u8;
        self.write_reg(ata_regs::ATA_REG_DEVICE, device);

        // Set sector count
        self.write_reg(ata_regs::ATA_REG_NSECT, count as u8);

        // Set LBA
        self.write_reg(ata_regs::ATA_REG_LBAL, (start_lba & 0xFF) as u8);
        self.write_reg(ata_regs::ATA_REG_LBAM, ((start_lba >> 8) & 0xFF) as u8);
        self.write_reg(ata_regs::ATA_REG_LBAH, ((start_lba >> 16) & 0xFF) as u8);

        // Send READ command
        self.write_reg(ata_regs::ATA_REG_CMD, ata_cmd::ATA_CMD_READ_PIO);

        // Read data
        let mut data_port: Port<u16> = Port::new(self.base_port + ata_regs::ATA_REG_DATA);
        let mut buf_offset = 0usize;
        let mut total_read = 0usize;

        for _sector in 0..count {
            // Wait for DRQ
            self.wait_drq(ata_timeouts::DRQ_TIMEOUT)?;

            // Read 256 words (512 bytes) per sector
            for _word in 0..ata_identify::WORDS_PER_SECTOR {
                let word = unsafe { data_port.read() };
                if buf_offset < buf.len() {
                    buf[buf_offset] = (word & 0xFF) as u8;
                    buf_offset += 1;
                }
                if buf_offset < buf.len() {
                    buf[buf_offset] = ((word >> 8) & 0xFF) as u8;
                    buf_offset += 1;
                }
                total_read += 2;
            }
        }

        // Check status
        self.wait_not_busy()?;
        let status = self.read_reg(ata_regs::ATA_REG_STATUS);
        if status & ata_status::ATA_ERR != 0 {
            let error = self.read_reg(ata_regs::ATA_REG_ERR);
            self.stats.read_errors += 1;
            serial::write_string("ATA: Read error: ");
            serial::write_hex(error as u64);
            serial::write_string("\n");
            return Err(KernelError::IoError);
        }

        self.stats.reads += 1;
        self.stats.read_sectors += count as u64;

        Ok(total_read)
    }

    /// Write sectors using PIO (Linux: ata_pio_write_sectors)
    fn write_sectors_pio(
        &mut self,
        start_lba: u64,
        count: usize,
        buf: &[u8],
    ) -> KernelResult<usize> {
        let expected_bytes = count * self.sector_size;
        if buf.len() < expected_bytes {
            return Err(KernelError::BufferTooSmall);
        }

        // Wait for device
        self.wait_not_busy()?;

        // Select device with LBA
        let device = match self.drive {
            AtaDrive::Master => ata_device::ATA_MASTER_MAGIC,
            AtaDrive::Slave => ata_device::ATA_SLAVE_MAGIC,
        } | ata_device::ATA_LBA
            | ((start_lba >> 24) & 0x0F) as u8;
        self.write_reg(ata_regs::ATA_REG_DEVICE, device);

        // Set sector count
        self.write_reg(ata_regs::ATA_REG_NSECT, count as u8);

        // Set LBA
        self.write_reg(ata_regs::ATA_REG_LBAL, (start_lba & 0xFF) as u8);
        self.write_reg(ata_regs::ATA_REG_LBAM, ((start_lba >> 8) & 0xFF) as u8);
        self.write_reg(ata_regs::ATA_REG_LBAH, ((start_lba >> 16) & 0xFF) as u8);

        // Send WRITE command
        self.write_reg(ata_regs::ATA_REG_CMD, ata_cmd::ATA_CMD_WRITE_PIO);

        // Write data
        let mut data_port: Port<u16> = Port::new(self.base_port + ata_regs::ATA_REG_DATA);
        let mut buf_offset = 0usize;

        for _sector in 0..count {
            // Wait for DRQ
            self.wait_drq(ata_timeouts::DRQ_TIMEOUT)?;

            // Write 256 words (512 bytes) per sector
            for _word in 0..ata_identify::WORDS_PER_SECTOR {
                let byte0 = buf.get(buf_offset).copied().unwrap_or(0);
                buf_offset += 1;
                let byte1 = buf.get(buf_offset).copied().unwrap_or(0);
                buf_offset += 1;
                let word = (byte1 as u16) << 8 | (byte0 as u16);
                unsafe { data_port.write(word) };
            }
        }

        // Wait for completion
        self.wait_not_busy()?;

        // Send flush cache command
        self.write_reg(ata_regs::ATA_REG_CMD, ata_cmd::ATA_CMD_FLUSH_CACHE);
        self.wait_not_busy()?;

        // Check status
        let status = self.read_reg(ata_regs::ATA_REG_STATUS);
        if status & ata_status::ATA_ERR != 0 {
            let error = self.read_reg(ata_regs::ATA_REG_ERR);
            self.stats.write_errors += 1;
            serial::write_string("ATA: Write error: ");
            serial::write_hex(error as u64);
            serial::write_string("\n");
            return Err(KernelError::IoError);
        }

        self.stats.writes += 1;
        self.stats.write_sectors += count as u64;

        Ok(expected_bytes)
    }
}

/// BlockDevice implementation for ATA device
impl BlockDevice for AtaDevice {
    fn read_sectors(&mut self, sector: u64, count: usize, buf: &mut [u8]) -> KernelResult<usize> {
        if self.state != AtaState::Identified {
            return Err(KernelError::DeviceNotReady);
        }
        self.read_sectors_pio(sector, count, buf)
    }

    fn write_sectors(&mut self, sector: u64, count: usize, buf: &[u8]) -> KernelResult<usize> {
        if self.state != AtaState::Identified {
            return Err(KernelError::DeviceNotReady);
        }
        self.write_sectors_pio(sector, count, buf)
    }

    fn num_sectors(&self) -> u64 {
        self.num_sectors
    }

    fn sector_size(&self) -> usize {
        self.sector_size
    }

    fn is_present(&self) -> bool {
        self.state == AtaState::Identified && self.num_sectors > 0
    }

    fn device_name(&self) -> &str {
        "hda"
    }
}

/// Global ATA device handles (Linux: ide_hwif_t)
static mut ATA_PRIMARY_MASTER: Option<Arc<Mutex<dyn BlockDevice>>> = None;

/// Initialize the ATA driver (Linux: ata_init)
pub fn init() -> KernelResult<()> {
    serial::write_string("ATA: Initializing PATA driver\n");

    // Try to detect primary master
    serial::write_string("ATA: Creating device...\n");
    let mut primary_master = AtaDevice::new(AtaChannel::Primary, AtaDrive::Master);
    serial::write_string("ATA: Calling identify...\n");

    // Try to identify the device - this may fail if no device is present
    match primary_master.identify() {
        Ok(()) => {
            serial::write_string("ATA: Identify OK\n");
            // Now that memory is ready, store globally
            let arc_dev: Arc<Mutex<dyn BlockDevice>> = Arc::new(Mutex::new(primary_master));
            unsafe {
                ATA_PRIMARY_MASTER = Some(arc_dev);
            }
            serial::write_string("ATA: Primary master detected and stored.\n");
        }
        Err(_e) => {
            serial::write_string("ATA: No primary master device\n");
            // Don't return error - ATA is optional
        }
    }

    Ok(())
}

/// Get the primary master device
pub fn get_primary_master() -> Option<Arc<Mutex<dyn BlockDevice>>> {
    unsafe { ATA_PRIMARY_MASTER.clone() }
}
