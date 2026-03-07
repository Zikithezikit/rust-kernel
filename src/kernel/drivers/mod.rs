//! Hardware drivers module
//!
//! This module groups together all device drivers:
//! - VGA text mode display driver
//! - Serial port driver
//! - Block device interface
//! - ATA/PATA disk driver

pub mod ata;
pub mod block;
pub mod init;
pub mod serial;
pub mod vga;
