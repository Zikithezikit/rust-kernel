//! ELF loader implementation
//!
//! Provides functionality to parse and load ELF64 binaries into memory.

pub mod elf;
pub mod loader;

use crate::include::error::KernelResult;
use crate::arch::x86::regs::PtRegs;

/// Load and execute a binary
pub fn exec_binary(path: &str, regs: &mut PtRegs) -> KernelResult<()> {
    loader::load_and_run(path, regs)
}
