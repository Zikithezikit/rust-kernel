//! Memory management module (mm)
//!
//! This module contains memory management similar to Linux mm/:
//! - Physical Memory Manager (PMM)
//! - Virtual Memory Manager (VMM)
//! - Heap/Slab allocator
//! - Page table management

pub mod allocator;
pub mod pmm;
pub mod vmm;
