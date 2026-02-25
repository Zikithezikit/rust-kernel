//! Kernel initialization module
//!
//! Handles early kernel setup including PMM and heap initialization.

use crate::allocator::PmmAllocator;
use crate::pmm::PMM;

/// Initializes the PMM with memory information from Multiboot2
///
/// This must be called early in boot to enable memory allocation.
///
/// # Arguments
/// * `multiboot_info` - Physical pointer to Multiboot2 structure
///
/// # Returns
/// `true` if initialization succeeded, `false` otherwise
pub fn init_pmm(multiboot_info: usize) -> bool {
    if multiboot_info == 0 {
        crate::std_lib::serial::write_string("Error: Invalid Multiboot2 pointer\n");
        return false;
    }

    unsafe { PMM.init(multiboot_info) };

    // Check if PMM was initialized successfully
    if PMM.get_total_pages() == 0 {
        crate::std_lib::serial::write_string("Error: PMM initialization failed\n");
        return false;
    }

    true
}

/// Initializes the heap allocator
///
/// Must be called after PMM is initialized.
///
/// # Arguments
/// * `allocator` - The global allocator instance
/// * `heap_start` - Starting address of heap
/// * `heap_size` - Size of heap region
///
/// # Returns
/// `true` if initialization succeeded, `false` otherwise
pub fn init_heap(allocator: &PmmAllocator, heap_start: usize, heap_size: usize) -> bool {
    if heap_start == 0 || heap_size == 0 {
        crate::std_lib::serial::write_string("Error: Invalid heap parameters\n");
        return false;
    }

    unsafe { allocator.init(heap_start, heap_size) };
    true
}

/// Early kernel initialization
///
/// Called from kernel_main to set up memory management.
///
/// # Arguments
/// * `multiboot_info` - Physical pointer to Multiboot2 structure
/// * `allocator` - The global allocator instance
///
/// # Returns
/// `true` if all initialization succeeded, `false` otherwise
pub fn init_memory(multiboot_info: usize, allocator: &PmmAllocator) -> bool {
    const HEAP_START: usize = 0x_100_000;
    const HEAP_SIZE: usize = 0x_100_000;

    if !init_pmm(multiboot_info) {
        return false;
    }

    init_heap(allocator, HEAP_START, HEAP_SIZE)
}
