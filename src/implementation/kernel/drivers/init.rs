//! Driver initialization module
//!
//! Handles initialization of hardware drivers: VGA and serial.

use crate::drivers::serial;
use crate::drivers::vga;

#[derive(Debug)]
pub enum DriverError {
    VgaInitFailed,
    SerialInitFailed,
}

impl core::fmt::Display for DriverError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DriverError::VgaInitFailed => write!(f, "VGA initialization failed"),
            DriverError::SerialInitFailed => write!(f, "Serial initialization failed"),
        }
    }
}

pub fn init_vga() -> Result<(), DriverError> {
    vga::clear_screen();
    vga::println("Starting...");
    Ok(())
}

pub fn init_serial() -> Result<(), DriverError> {
    unsafe {
        serial::init();
    }
    serial::write_string("Kernel started\n");
    Ok(())
}

pub fn init() -> Result<(), DriverError> {
    init_vga()?;
    init_serial()
}
