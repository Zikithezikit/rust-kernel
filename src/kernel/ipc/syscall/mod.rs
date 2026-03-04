//! System call module
//!
//! Provides system call interface for user programs.
//! Uses int 0x80 for syscall invocation on x86.

pub mod handler;
pub mod numbers;
pub mod syscall;
