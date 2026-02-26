//! Memory management module
//!
//! This module groups together all memory-related functionality:
//! - Physical Memory Manager (PMM)
//! - Virtual Memory Manager (VMM)  
//! - Heap allocator
//! - Memory initialization

pub mod allocator;
pub mod globals;
pub mod init;
pub mod pmm;
pub mod vmm;
