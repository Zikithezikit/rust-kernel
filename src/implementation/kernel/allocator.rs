use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use spin::Mutex;

/// Bump allocator tracks a contiguous region of memory and hands out pieces one after another.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
}

/// Global lock protecting the bump allocator state.
static HEAP: Mutex<Option<Bump>> = Mutex::new(None);

/// Tracks the current position in the heap where the next allocation will begin.
struct Bump {
    next: usize,
}

impl BumpAllocator {
    /// Creates a new bump allocator with the given memory region.
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        BumpAllocator {
            heap_start,
            heap_end,
        }
    }

    /// Initializes the allocator by setting the heap start position.
    pub fn init(&self) {
        *HEAP.lock() = Some(Bump {
            next: self.heap_start,
        });
    }
}

/// Implements the Rust global allocator trait for our bump allocator.
unsafe impl GlobalAlloc for BumpAllocator {
    /// Allocates memory by moving a pointer forward. Simple but cannot reuse freed memory.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = HEAP.lock();
        if let Some(ref mut bump) = *heap {
            let align = layout.align();
            let size = layout.size();

            // Round up to meet alignment requirements.
            let aligned_next = (bump.next + align - 1) & !(align - 1);

            // Check if we have enough space left in the heap.
            if aligned_next + size > self.heap_end {
                ptr::null_mut()
            } else {
                bump.next = aligned_next + size;
                aligned_next as *mut u8
            }
        } else {
            ptr::null_mut()
        }
    }

    /// Bump allocator does not reuse memory, so dealloc is a no-op.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
