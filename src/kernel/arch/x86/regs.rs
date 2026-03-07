//! Register state preservation
//!
//! Defines the PtRegs structure which holds the CPU register state
//! when entering the kernel from user mode.

/// Register state pushed on the stack during a syscall or interrupt
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct PtRegs {
    // General purpose registers pushed by assembly
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,

    /// Error code (dummy for syscalls, real for some exceptions)
    pub error_code: u64,

    // The following are pushed automatically by the CPU on interrupt
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl PtRegs {
    /// Create an empty register state
    pub fn new() -> Self {
        Self::default()
    }
}
