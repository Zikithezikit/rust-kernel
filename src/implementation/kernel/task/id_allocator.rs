//! Task ID allocator
//!
//! Provides a bitmap-based ID allocator similar to Linux kernel's PID allocation.
//! IDs are reused after being freed, preventing overflow.

use core::sync::atomic::{AtomicUsize, Ordering};

/// Maximum number of concurrent tasks (Linux default is 32768)
pub const MAX_TASKS: usize = 32768;

/// Bits per usize
const BITS_PER_USIZE: usize = usize::BITS as usize;

/// Number of usize values needed for the bitmap
const BITMAP_ARRAY_SIZE: usize = (MAX_TASKS + BITS_PER_USIZE - 1) / BITS_PER_USIZE;

/// ID allocator using bitmap
pub struct IdAllocator {
    /// Bitmap tracking allocated IDs (1 = allocated, 0 = free)
    bitmap: [AtomicUsize; BITMAP_ARRAY_SIZE],
    /// Number of IDs currently allocated
    allocated: AtomicUsize,
    /// Next ID to try (hint for fast allocation)
    next_id: AtomicUsize,
}

impl IdAllocator {
    /// Creates a new ID allocator
    pub const fn new() -> Self {
        const EMPTY: AtomicUsize = AtomicUsize::new(0);
        IdAllocator {
            bitmap: [EMPTY; BITMAP_ARRAY_SIZE],
            allocated: AtomicUsize::new(0),
            next_id: AtomicUsize::new(1),
        }
    }

    /// Allocates a new unique ID
    ///
    /// # Returns
    /// - Some(id) - Newly allocated ID
    /// - None if no IDs available
    pub fn alloc(&self) -> Option<usize> {
        if self.allocated.load(Ordering::Relaxed) >= MAX_TASKS - 1 {
            return None;
        }

        let mut start = self.next_id.load(Ordering::Relaxed);
        if start >= MAX_TASKS {
            start = 1;
        }

        let mut id = start;
        loop {
            if self.try_allocate_id(id) {
                self.next_id.store(id + 1, Ordering::Relaxed);
                if self.next_id.load(Ordering::Relaxed) >= MAX_TASKS {
                    self.next_id.store(1, Ordering::Relaxed);
                }
                return Some(id);
            }

            id += 1;
            if id >= MAX_TASKS {
                id = 1;
            }

            if id == start {
                return None;
            }
        }
    }

    /// Tries to allocate a specific ID
    fn try_allocate_id(&self, id: usize) -> bool {
        if id == 0 || id >= MAX_TASKS {
            return false;
        }

        let idx = id / BITS_PER_USIZE;
        let bit = id % BITS_PER_USIZE;
        let mask = 1 << bit;

        let current = self.bitmap[idx].load(Ordering::Relaxed);
        if current & mask != 0 {
            return false;
        }

        if self.bitmap[idx]
            .compare_exchange(
                current,
                current | mask,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            self.allocated.fetch_add(1, Ordering::Relaxed);
            return true;
        }

        false
    }

    /// Frees an allocated ID, allowing it to be reused
    ///
    /// # Safety
    /// - The ID must have been previously allocated by this allocator
    /// - The ID must not be freed twice
    pub unsafe fn free(&self, id: usize) {
        if id == 0 || id >= MAX_TASKS {
            return;
        }

        let idx = id / BITS_PER_USIZE;
        let bit = id % BITS_PER_USIZE;
        let mask = 1 << bit;

        let current = self.bitmap[idx].load(Ordering::Relaxed);
        if current & mask != 0 {
            self.bitmap[idx].fetch_and(!mask, Ordering::Relaxed);
            self.allocated.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Returns the number of currently allocated IDs
    pub fn allocated_count(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Returns the maximum number of IDs
    pub fn max_ids(&self) -> usize {
        MAX_TASKS - 1
    }

    /// Checks if an ID is currently allocated
    pub fn is_allocated(&self, id: usize) -> bool {
        if id == 0 || id >= MAX_TASKS {
            return false;
        }

        let idx = id / BITS_PER_USIZE;
        let bit = id % BITS_PER_USIZE;
        let mask = 1 << bit;

        self.bitmap[idx].load(Ordering::Relaxed) & mask != 0
    }
}

/// Global task ID allocator
pub static TASK_ID_ALLOCATOR: IdAllocator = IdAllocator::new();
