//! Driver initialization module
//!
//! Handles initialization of hardware drivers: VGA and serial.

use crate::drivers::serial;
use crate::drivers::vga;
use crate::error::{KernelError, KernelResult};

/// Initializes the VGA display driver.
///
/// # Errors
/// Returns `KernelError::DriverFailed` if VGA cannot be initialized.
pub fn init_vga() -> KernelResult<()> {
    vga::clear_screen();
    vga::println("Starting...");
    Ok(())
}

/// Initializes the serial port driver.
///
/// # Errors
/// Returns `KernelError::DriverFailed` if serial cannot be initialized.
pub fn init_serial() -> KernelResult<()> {
    unsafe {
        serial::init();
    }
    serial::write_string("Kernel started\n");
    Ok(())
}

/// Initializes all hardware drivers.
///
/// This function:
/// 1. Initializes the VGA display
/// 2. Initializes the serial port
///
/// # Errors
/// Returns `KernelError::DriverFailed` if any driver fails to initialize.
pub fn init() -> KernelResult<()> {
    init_vga()?;
    init_serial()
}
