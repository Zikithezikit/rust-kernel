use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use spin::Mutex;

use crate::memory::pmm::PMM;

/// Bump allocator using PMM for heap memory
///
/// This allocator provides a simple bump-pointer heap on top of the PMM.
/// It allocates memory from a contiguous region and just moves a pointer forward.
/// This is fast but does not support freeing individual allocations.
pub struct PmmAllocator;

static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

/// Internal heap state
struct Heap {
    _start: usize,
    end: usize,
    current: usize,
}

impl PmmAllocator {
    /// Creates a new uninitialized heap allocator
    pub const fn new() -> Self {
        PmmAllocator
    }

    /// Initializes the heap allocator with a memory region
    ///
    /// # Arguments
    /// * `heap_start` - Starting address of heap
    /// * `heap_size` - Size of heap region
    ///
    /// # Safety
    /// The memory region must not overlap with any used memory
    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        if heap_start == 0 || heap_size == 0 {
            return;
        }

        let heap_end = heap_start + heap_size;
        *HEAP.lock() = Some(Heap {
            _start: heap_start,
            end: heap_end,
            current: heap_start,
        });
    }
}

/// Allocates physical pages using PMM (for future use)
#[allow(dead_code)]
pub unsafe fn allocate_pages(num_pages: usize) -> *mut u8 {
    let ptr = PMM.allocate_pages(num_pages);
    match ptr {
        Some(addr) => addr as *mut u8,
        None => ptr::null_mut(),
    }
}

/// Allocates a single physical page using PMM (for future use)
#[allow(dead_code)]
pub unsafe fn allocate_page() -> *mut u8 {
    let ptr = PMM.allocate_page();
    match ptr {
        Some(addr) => addr as *mut u8,
        None => ptr::null_mut(),
    }
}

/// Frees physical pages (for future use)
#[allow(dead_code)]
pub unsafe fn free_pages(addr: usize, num_pages: usize) {
    PMM.deallocate_pages(addr, num_pages);
}

unsafe impl GlobalAlloc for PmmAllocator {
    /// Allocates memory from the heap region
    ///
    /// Uses bump-pointer allocation - fast but cannot reuse freed memory.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = HEAP.lock();
        if let Some(ref mut heap) = *heap {
            let align = layout.align();
            let size = layout.size();

            // Round up to meet alignment requirements
            let aligned_current = (heap.current + align - 1) & !(align - 1);

            // Check if we have enough space left in the heap
            if aligned_current + size > heap.end {
                ptr::null_mut()
            } else {
                heap.current = aligned_current + size;
                aligned_current as *mut u8
            }
        } else {
            // Heap not initialized - allocation fails
            ptr::null_mut()
        }
    }

    /// Frees memory
    ///
    /// Bump allocator does not support freeing - this is a no-op.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator cannot free individual allocations
    }
}
