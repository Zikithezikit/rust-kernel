//! Kernel Heap Allocator
//!
//! This module provides the global heap allocator for the kernel.
//! Currently implements a bump allocator that supports both allocation and deallocation
//! through page-level tracking.
//!
//! ## Overview
//!
//! The allocator uses a tiered approach:
//! 1. **Bump allocator**: For small allocations
//! 2. **Page allocator**: For larger allocations
//!
//! This is a simplified implementation that works reliably in kernel context.

use alloc::format;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use spin::Mutex;

use crate::mm::pmm::PMM;

/// Page size (4KB)
const PAGE_SIZE: usize = 4096;

/// Wrapper for raw pointers to make them Send-safe in single-threaded kernel
///
/// # Safety
/// In a single-threaded kernel context, raw pointers are safe to pass between
/// "threads" (the mutex guard) because there's only one execution context.
struct SendPtr(*mut u8);

// SAFETY: In single-threaded kernel, we manually ensure only one access at a time
unsafe impl Send for SendPtr {}

/// Global bump allocator state - protected by mutex for thread-safe access
static BUMP_ALLOCATOR: Mutex<BumpAllocator> = Mutex::new(BumpAllocator::new());

/// Bump allocator with page-level tracking for freeing
pub struct BumpAllocator {
    /// Start of heap region
    start: usize,
    /// Current allocation pointer
    current: usize,
    /// End of heap region
    end: usize,
    /// List of allocated pages for tracking
    allocated_pages: Mutex<Vec<(SendPtr, usize)>>,
}

impl BumpAllocator {
    /// Create a new uninitialized bump allocator
    pub const fn new() -> Self {
        BumpAllocator {
            start: 0,
            current: 0,
            end: 0,
            allocated_pages: Mutex::new(Vec::new()),
        }
    }

    /// Initialize the allocator
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        if heap_start == 0 || heap_size == 0 {
            return;
        }

        self.start = heap_start;
        self.current = heap_start;
        self.end = heap_start + heap_size;
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.start != 0
    }

    /// Allocate memory
    ///
    /// # Safety
    /// Caller must ensure proper alignment and size
    pub unsafe fn allocate(&self, layout: Layout) -> *mut u8 {
        if !self.is_initialized() {
            return ptr::null_mut();
        }

        let size = layout.size();
        let align = layout.align();

        // Handle large allocations via page allocator
        if size > PAGE_SIZE / 2 {
            let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
            if let Some(addr) = PMM.allocate_pages(pages) {
                let ptr = addr as *mut u8;
                // Track for potential freeing
                self.allocated_pages.lock().push((SendPtr(ptr), pages));
                return ptr;
            }
            return ptr::null_mut();
        }

        // For small allocations, we need to use atomic operations or a lock
        // Since we can't mutate self with &self, use a simple approach:
        // Use atomic for current pointer
        use core::sync::atomic::{AtomicUsize, Ordering};
        static CURRENT_PTR: AtomicUsize = AtomicUsize::new(0);

        // Initialize on first use
        if CURRENT_PTR.load(Ordering::Relaxed) == 0 {
            CURRENT_PTR.store(self.start, Ordering::Relaxed);
        }

        loop {
            let current = CURRENT_PTR.load(Ordering::Relaxed);

            // Align
            let aligned = (current + align - 1) & !(align - 1);

            // Check bounds
            if aligned + size > self.end {
                return ptr::null_mut();
            }

            // Try to claim this space atomically
            let new_current = aligned + size;
            if CURRENT_PTR
                .compare_exchange(current, new_current, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return aligned as *mut u8;
            }
            // Retry if another thread/modifier got there first
        }
    }

    /// Free memory
    ///
    /// Note: For simplicity, we track large allocations but don't actually
    /// return small allocations to the pool. This is a known limitation.
    pub unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() || !self.is_initialized() {
            return;
        }

        let size = layout.size();

        // For large allocations, we can free pages
        if size > PAGE_SIZE / 2 {
            let _pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
            let addr = ptr as usize;

            // Find and remove from tracking
            let mut pages_lock = self.allocated_pages.lock();
            if let Some(pos) = pages_lock.iter().position(|(p, _)| p.0 == ptr) {
                let (_, pcount) = pages_lock.remove(pos);
                PMM.deallocate_pages(addr, pcount);
            }
        }
        // Small allocations are not actually freed (bump allocator limitation)
    }
}

// Global allocator implementation
pub struct PmmAllocator;

impl PmmAllocator {
    /// Create a new PmmAllocator
    pub const fn new() -> Self {
        PmmAllocator
    }

    /// Initialize the allocator
    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        let mut allocator = BUMP_ALLOCATOR.lock();
        allocator.init(heap_start, heap_size);
    }

    /// Check if initialized
    pub fn is_initialized() -> bool {
        BUMP_ALLOCATOR.lock().is_initialized()
    }
}

unsafe impl GlobalAlloc for PmmAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        BUMP_ALLOCATOR.lock().allocate(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        BUMP_ALLOCATOR.lock().deallocate(ptr, layout)
    }
}

// Legacy API
#[allow(dead_code)]
pub unsafe fn allocate_pages(num_pages: usize) -> *mut u8 {
    match PMM.allocate_pages(num_pages) {
        Some(addr) => addr as *mut u8,
        None => ptr::null_mut(),
    }
}

#[allow(dead_code)]
pub unsafe fn allocate_page() -> *mut u8 {
    match PMM.allocate_page() {
        Some(addr) => addr as *mut u8,
        None => ptr::null_mut(),
    }
}

#[allow(dead_code)]
pub unsafe fn free_pages(addr: usize, num_pages: usize) {
    PMM.deallocate_pages(addr, num_pages);
}
