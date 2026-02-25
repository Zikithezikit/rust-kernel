use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use spin::Mutex;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
}

static HEAP: Mutex<Option<Bump>> = Mutex::new(None);

struct Bump {
    next: usize,
}

impl BumpAllocator {
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        BumpAllocator {
            heap_start,
            heap_end,
        }
    }

    pub fn init(&self) {
        *HEAP.lock() = Some(Bump {
            next: self.heap_start,
        });
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = HEAP.lock();
        if let Some(ref mut bump) = *heap {
            let align = layout.align();
            let size = layout.size();

            let aligned_next = (bump.next + align - 1) & !(align - 1);

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

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
