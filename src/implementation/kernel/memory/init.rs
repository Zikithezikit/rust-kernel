//! Kernel initialization module
//!
//! Handles early kernel setup including PMM and heap initialization.

use crate::drivers::serial;
use crate::memory::allocator::PmmAllocator;
use crate::memory::pmm::PMM;

const HEAP_START: usize = 0x_100_000;
const HEAP_SIZE: usize = 0x_100_000;

#[derive(Debug)]
pub enum MemoryError {
    InvalidMultibootInfo,
    PmmInitFailed,
    InvalidHeapParams,
}

impl core::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MemoryError::InvalidMultibootInfo => write!(f, "Invalid Multiboot2 info pointer"),
            MemoryError::PmmInitFailed => write!(f, "PMM initialization failed"),
            MemoryError::InvalidHeapParams => write!(f, "Invalid heap parameters"),
        }
    }
}

pub fn init_pmm(multiboot_info: usize) -> Result<(), MemoryError> {
    if multiboot_info == 0 {
        serial::write_string("Error: Invalid Multiboot2 pointer\n");
        return Err(MemoryError::InvalidMultibootInfo);
    }

    unsafe { PMM.init(multiboot_info) };

    if PMM.get_total_pages() == 0 {
        serial::write_string("Error: PMM initialization failed\n");
        return Err(MemoryError::PmmInitFailed);
    }

    Ok(())
}

pub fn init_heap(
    allocator: &PmmAllocator,
    heap_start: usize,
    heap_size: usize,
) -> Result<(), MemoryError> {
    if heap_start == 0 || heap_size == 0 {
        serial::write_string("Error: Invalid heap parameters\n");
        return Err(MemoryError::InvalidHeapParams);
    }

    unsafe { allocator.init(heap_start, heap_size) };
    Ok(())
}

pub fn init_memory(multiboot_info: usize, allocator: &PmmAllocator) -> Result<(), MemoryError> {
    init_pmm(multiboot_info)?;
    init_heap(allocator, HEAP_START, HEAP_SIZE)
}
