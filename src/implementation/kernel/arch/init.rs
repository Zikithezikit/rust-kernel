//! Architecture initialization module
//!
//! Handles architecture-specific setup including TSS, IDT, PIC, PIT, and interrupts.

use crate::arch::x86::interrupts;
use crate::arch::x86::tss;
use crate::error::{KernelError, KernelResult};

/// Initializes the TSS (Task State Segment).
///
/// # Errors
/// Returns `KernelError::TssInitFailed` if TSS initialization fails.
pub fn init_tss() -> KernelResult<()> {
    tss::init()
}

/// Loads the TSS into the CPU.
///
/// # Errors
/// Returns `KernelError::TssInitFailed` if TSS loading fails.
pub fn load_tss() -> KernelResult<()> {
    tss::load()
}

/// Initializes all interrupt handling hardware and software.
///
/// This includes:
/// - PIC (Programmable Interrupt Controller)
/// - PIT (Programmable Interval Timer)
/// - IDT (Interrupt Descriptor Table)
///
/// # Errors
/// Returns `KernelError::IdtInitFailed` if interrupt initialization fails.
pub fn init_interrupts() -> KernelResult<()> {
    unsafe {
        interrupts::init_pic();
        interrupts::init_pit();
    }
    interrupts::init_idt();
    Ok(())
}

/// Enables CPU interrupts.
pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

/// Initializes all architecture-specific subsystems.
///
/// This function:
/// 1. Initializes the TSS
/// 2. Initializes interrupts (PIC, PIT, IDT)
/// 3. Loads the TSS
/// 4. Enables interrupts
///
/// # Errors
/// Returns `KernelError::TssInitFailed` or `KernelError::IdtInitFailed` if any step fails.
pub fn init_arch() -> KernelResult<()> {
    init_tss()?;
    init_interrupts()?;
    load_tss()?;
    enable_interrupts();
    Ok(())
}
