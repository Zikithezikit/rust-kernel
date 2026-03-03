//! Kernel initialization module
//!
//! Handles early kernel setup including PMM and heap initialization.

use crate::drivers::serial;
use crate::error::{KernelError, KernelResult};
use crate::memory::allocator::PmmAllocator;
use crate::memory::pmm::PMM;

const HEAP_START: usize = 0x_100_000;
const HEAP_SIZE: usize = 0x_100_000;

/// Initializes the Physical Memory Manager (PMM).
///
/// # Arguments
/// * `multiboot_info` - Multiboot2 information pointer from bootloader
///
/// # Errors
/// Returns `KernelError::MemoryInitFailed` if:
/// - Multiboot info pointer is invalid
/// - PMM initialization fails
pub fn init_pmm(multiboot_info: usize) -> KernelResult<()> {
    if multiboot_info == 0 {
        serial::write_string("Error: Invalid Multiboot2 pointer\n");
        return Err(KernelError::MemoryInitFailed("Invalid Multiboot2 pointer"));
    }

    unsafe { PMM.init(multiboot_info) };

    if PMM.get_total_pages() == 0 {
        serial::write_string("Error: PMM initialization failed\n");
        return Err(KernelError::MemoryInitFailed("PMM initialization failed"));
    }

    Ok(())
}

/// Initializes the kernel heap allocator.
///
/// # Arguments
/// * `allocator` - The allocator instance to initialize
/// * `heap_start` - Starting address of the heap
/// * `heap_size` - Size of the heap in bytes
///
/// # Errors
/// Returns `KernelError::MemoryInitFailed` if heap parameters are invalid.
pub fn init_heap(
    allocator: &PmmAllocator,
    heap_start: usize,
    heap_size: usize,
) -> KernelResult<()> {
    if heap_start == 0 || heap_size == 0 {
        serial::write_string("Error: Invalid heap parameters\n");
        return Err(KernelError::MemoryInitFailed("Invalid heap parameters"));
    }

    unsafe { allocator.init(heap_start, heap_size) };
    Ok(())
}

/// Initializes all memory subsystems.
///
/// This function:
/// 1. Initializes the Physical Memory Manager
/// 2. Initializes the kernel heap
///
/// # Arguments
/// * `multiboot_info` - Multiboot2 information pointer
/// * `allocator` - The heap allocator instance
///
/// # Errors
/// Returns `KernelError::MemoryInitFailed` if any step fails.
pub fn init_memory(multiboot_info: usize, allocator: &PmmAllocator) -> KernelResult<()> {
    init_pmm(multiboot_info)?;
    init_heap(allocator, HEAP_START, HEAP_SIZE)
}
