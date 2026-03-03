//! Driver initialization module
//!
//! Handles initialization of hardware drivers: VGA and serial.

use crate::drivers::serial;
use crate::error::KernelResult;

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

/// Initializes all hardware drivers.
///
/// This function:
/// 1. Initializes the serial port
///
/// # Errors
/// Returns `KernelError::DriverFailed` if any driver fails to initialize.
pub fn init() -> KernelResult<()> {
    init_serial()?;
    Ok(())
}
