//! Architecture initialization module
//!
//! Handles architecture-specific setup including TSS, IDT, PIC, PIT, and interrupts.

use crate::arch::x86::interrupts;
use crate::arch::x86::tss;
use crate::drivers::serial;

pub use crate::arch::x86::tss::TssError as ArchError;

pub fn init_tss() -> Result<(), ArchError> {
    tss::init()
}

pub fn load_tss() -> Result<(), ArchError> {
    tss::load()
}

pub fn init_interrupts() -> Result<(), ArchError> {
    unsafe {
        interrupts::init_pic();
        interrupts::init_pit();
    }
    interrupts::init_idt();
    Ok(())
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

pub fn init_arch() -> Result<(), ArchError> {
    init_tss()?;
    init_interrupts()?;
    load_tss()?;
    enable_interrupts();
    Ok(())
}
