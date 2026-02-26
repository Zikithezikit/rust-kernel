//! Kernel initialization module
//!
//! Handles early kernel setup including PMM and heap initialization.

use crate::allocator::PmmAllocator;
use crate::drivers::serial;
use crate::memory::pmm::PMM;

pub fn init_pmm(multiboot_info: usize) -> bool {
    if multiboot_info == 0 {
        serial::write_string("Error: Invalid Multiboot2 pointer\n");
        return false;
    }

    unsafe { PMM.init(multiboot_info) };

    if PMM.get_total_pages() == 0 {
        serial::write_string("Error: PMM initialization failed\n");
        return false;
    }

    true
}

pub fn init_heap(allocator: &PmmAllocator, heap_start: usize, heap_size: usize) -> bool {
    if heap_start == 0 || heap_size == 0 {
        serial::write_string("Error: Invalid heap parameters\n");
        return false;
    }

    unsafe { allocator.init(heap_start, heap_size) };
    true
}

pub fn init_memory(multiboot_info: usize, allocator: &PmmAllocator) -> bool {
    const HEAP_START: usize = 0x_100_000;
    const HEAP_SIZE: usize = 0x_100_000;

    if !init_pmm(multiboot_info) {
        return false;
    }

    init_heap(allocator, HEAP_START, HEAP_SIZE)
}
