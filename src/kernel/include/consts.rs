//! Kernel-wide constants
//!
//! Centralized location for magic numbers and configuration values.

/// Page size in bytes
pub const PAGE_SIZE: usize = 4096;

/// Default user stack location
pub const USER_STACK_TOP: u64 = 0x7000_0000_0000;

/// Default user stack size in pages
pub const USER_STACK_PAGES: usize = 16;

/// User mode privilege level (Ring 3)
pub const USER_RING: u8 = 3;

/// Default RFLAGS for user mode (Interrupts enabled, bit 1 set)
pub const USER_RFLAGS: u64 = 0x202;

/// Maximum length for a file path in system calls
pub const MAX_PATH_LEN: usize = 256;

/// ELF Machine type for x86_64
pub const ELF_MACHINE_X86_64: u16 = 0x3E;

/// ELF type for executable file
pub const ELF_TYPE_EXEC: u16 = 2;

/// Number of registers saved/restored by context_switch
pub const CONTEXT_SWITCH_REGS: usize = 6;

/// Logging interval for syscalls in ticks
pub const SYSCALL_LOG_INTERVAL: u64 = 100;
