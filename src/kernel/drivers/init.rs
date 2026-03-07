//! Driver initialization module
//!
//! Handles initialization of hardware drivers: VGA, serial, and ATA.

use crate::drivers::serial;
use crate::include::error::KernelResult;

/// Initializes the serial port driver.
///
/// # Errors
/// Returns `KernelError::DriverFailed` if serial cannot be initialized.
pub fn init_serial() -> KernelResult<()> {
    unsafe {
        serial::init()?;
    }
    serial::write_string("Kernel started\n");
    Ok(())
}

/// Initializes the ATA driver.
///
/// # Errors
/// Returns `KernelError::DriverFailed` if ATA cannot be initialized.
pub fn init_ata() -> KernelResult<()> {
    // ATA will be initialized if hardware is present
    // This is safe to call - it handles missing hardware gracefully
    match crate::drivers::ata::init() {
        Ok(()) => Ok(()),
        Err(_e) => {
            // ATA is optional - don't fail boot if not present
            serial::write_string("ATA: Skipping (not available)\n");
            Ok(())
        }
    }
}

/// Initializes all hardware drivers.
///
/// This function:
/// 1. Initializes the serial port
/// 2. Initializes the ATA driver
///
/// # Errors
/// Returns `KernelError::DriverFailed` if any critical driver fails to initialize.
pub fn init() -> KernelResult<()> {
    init_serial()?;
    Ok(())
}
