//! Driver initialization module
//!
//! Handles initialization of hardware drivers: VGA and serial.

use crate::drivers::serial;
use crate::drivers::vga;

pub fn init_vga() -> bool {
    vga::clear_screen();
    vga::println("Starting...");
    true
}

pub fn init_serial() -> bool {
    unsafe {
        serial::init();
    }
    serial::write_string("Kernel started\n");
    true
}

pub fn init() -> bool {
    if !init_vga() {
        return false;
    }

    if !init_serial() {
        return false;
    }

    true
}
