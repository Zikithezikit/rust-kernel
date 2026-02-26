//! Physical Memory Manager (PMM)
//!
//! This module implements a bitmap-based physical memory allocator using the
//! Multiboot2 information structure provided by GRUB to detect available memory.
//!
//! ## Overview
//!
//! The PMM uses a bitmap to track which physical memory pages are allocated vs free.
//! Each bit represents one 4KB page. When a page is allocated, its bit is set to 1.
//!
//! ## Memory Detection
//!
//! On boot, the kernel receives a Multiboot2 information structure from GRUB containing
//! a memory map (tag type 6). Each entry describes a memory region with:
//! - base_addr: starting physical address
//! - length: size of the region  
//! - type: 1 = available, 2+ = reserved
//!
//! ## Usage
//!
//! ```rust
//! // Initialize PMM with multiboot info pointer (passed from boot assembly)
//! unsafe { PMM.init(multiboot_info_ptr); }
//!
//! // Allocate a single page (returns physical address or None)
//! let page = PMM.allocate_page();
//!
//! // Allocate multiple contiguous pages
//! let pages = PMM.allocate_pages(4);
//!
//! // Free pages when done
//! PMM.deallocate_page(page_addr);
//! ```

use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

/// Size of a physical memory page in bytes (4KB)
pub const PAGE_SIZE: usize = 4096;

/// Number of bits in a byte (for bitmap calculations)
const BITS_PER_BYTE: usize = 8;

/// Maximum physical memory pages we track (4GB worth)
const MAX_PHYSICAL_PAGES: usize = 0x100000;
/// Maximum pages the bitmap can track
const MAX_BITMAP_PAGES: usize = 64;
/// Kernel load address (1MB)
const KERNEL_START: usize = 0x100000;
/// Kernel size to reserve (2MB)
const KERNEL_SIZE: usize = 0x200000;

/// Multiboot2 parser module
///
/// Provides structures for parsing the Multiboot2 information structure passed by GRUB.
pub mod multiboot {
    /// Multiboot2 tag type identifiers
    ///
    /// Values from Multiboot2 specification:
    /// https://www.gnu.org/software/grub/manual/multiboot2/html_node/Boot-information-format.html
    #[derive(Debug, Clone, Copy)]
    #[allow(dead_code)]
    pub enum TagType {
        End = 0,
        BasicMemoryInfo = 4,
        BiosBootDevice = 5,
        MemoryMap = 6,
        VbeInfo = 7,
        FramebufferInfo = 8,
        ElfSymbols = 9,
        ApmTable = 10,
        Efi32Info = 11,
        Efi64Info = 12,
        BootLoaderName = 16,
        CommandLine = 17,
        BootModules = 18,
        EfiMmap = 20,
        EfiBootServicesNotUsed = 21,
        Efi32ImageHandle = 22,
        Efi64ImageHandle = 23,
        ImageLoadPhysAddrRange = 24,
    }

    /// Memory map entry from Multiboot2
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct MemoryMapEntry {
        pub entry_size: u32,
        pub entry_version: u32,
        pub base_addr: u64,
        pub length: u64,
        pub ty: u32,
    }

    /// Memory type: available for OS use
    pub const MEMORY_AVAILABLE: u32 = 1;

    /// Parser for the Multiboot2 information structure
    ///
    /// The Multiboot2 info structure is passed by GRUB in the EBX register.
    /// It starts with a 32-bit total size, followed by tags.
    pub struct Multiboot2Info {
        ptr: usize,
        total_size: u32,
    }

    impl Multiboot2Info {
        pub unsafe fn new(ptr: usize) -> Option<Self> {
            if ptr == 0 {
                return None;
            }
            let total_size = *(ptr as *const u32);
            if total_size == 0 {
                return None;
            }
            Some(Multiboot2Info { ptr, total_size })
        }

        pub fn tags(&self) -> Multiboot2TagIter {
            Multiboot2TagIter {
                ptr: self.ptr + 8,
                end: self.ptr + self.total_size as usize,
            }
        }
    }

    pub struct Multiboot2TagIter {
        ptr: usize,
        end: usize,
    }

    impl Iterator for Multiboot2TagIter {
        type Item = Multiboot2Tag;

        fn next(&mut self) -> Option<Self::Item> {
            if self.ptr >= self.end {
                return None;
            }

            unsafe {
                let ty = *(self.ptr as *const u16);
                let size = *((self.ptr + 4) as *const u32);

                if ty == 0 && size == 8 {
                    return None;
                }

                let result = Some(Multiboot2Tag {
                    ptr: self.ptr,
                    ty,
                    size,
                });

                let aligned_size = ((size + 7) / 8) * 8;
                self.ptr += aligned_size as usize;

                result
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Multiboot2Tag {
        ptr: usize,
        ty: u16,
        size: u32,
    }

    impl Multiboot2Tag {
        pub fn ty(&self) -> u16 {
            self.ty
        }

        #[allow(dead_code)]
        pub fn size(&self) -> u32 {
            self.size
        }

        pub fn memory_map_entries(&self) -> Option<MemoryMapIter> {
            if self.ty != TagType::MemoryMap as u16 {
                return None;
            }

            Some(MemoryMapIter {
                ptr: self.ptr + 8,
                offset: 0,
                entry_size: 24,
                end: self.ptr + self.size as usize,
            })
        }
    }

    pub struct MemoryMapIter {
        ptr: usize,
        offset: usize,
        entry_size: u32,
        end: usize,
    }

    impl Iterator for MemoryMapIter {
        type Item = MemoryMapEntry;

        fn next(&mut self) -> Option<Self::Item> {
            if self.offset >= self.end {
                return None;
            }

            unsafe {
                let entry = *(self.ptr as *const MemoryMapEntry);
                self.offset += self.entry_size as usize;
                self.ptr += self.entry_size as usize;
                Some(entry)
            }
        }
    }
}

/// Physical Memory Manager
///
/// Manages physical memory allocation using a bitmap approach.
/// Each physical page (4KB) is represented by one bit in the bitmap.
///
/// ## Initialization
///
/// The PMM must be initialized with a valid Multiboot2 information pointer:
/// ```rust
/// unsafe { PMM.init(multiboot_info_ptr); }
/// ```
///
/// ## Allocation
///
/// ```rust
/// // Allocate single page
/// let addr = PMM.allocate_page(); // Returns Option<usize>
///
/// // Allocate multiple contiguous pages
/// let addr = PMM.allocate_pages(4);
///
/// // Free pages
/// PMM.deallocate_page(addr.unwrap());
/// ```
pub struct PhysicalMemoryManager {
    bitmap: Mutex<Bitmap>,
    total_pages: AtomicUsize,
    reserved_pages: AtomicUsize,
}

/// Internal bitmap structure
///
/// The bitmap is a contiguous block of memory where each bit represents
/// a physical page. Bit = 0 means free, Bit = 1 means allocated.
struct Bitmap {
    /// Pointer to the bitmap data (allocated at boot)
    data: usize,
    /// Total number of pages this bitmap tracks
    num_pages: usize,
}

// Safety: Bitmap is managed by PMM which uses a Mutex for thread safety
unsafe impl Send for Bitmap {}
unsafe impl Sync for Bitmap {}

impl PhysicalMemoryManager {
    /// Creates a new uninitialized PMM
    pub const fn new() -> Self {
        PhysicalMemoryManager {
            bitmap: Mutex::new(Bitmap {
                data: 0,
                num_pages: 0,
            }),
            total_pages: AtomicUsize::new(0),
            reserved_pages: AtomicUsize::new(0),
        }
    }

    /// Initializes the PMM with memory information from Multiboot2
    ///
    /// This must be called before any memory allocation can occur.
    /// It parses the Multiboot2 memory map to determine available RAM.
    ///
    /// # Arguments
    /// * `multiboot_info_ptr` - Physical pointer to Multiboot2 info structure (from EBX register)
    ///
    /// # Process
    /// 1. Parse Multiboot2 structure to find memory map tag
    /// 2. Iterate through memory regions, tracking max address
    /// 3. Calculate bitmap size (1 bit per 4KB page)
    /// 4. Mark all pages as free initially
    /// 5. Reserve bitmap storage and kernel region
    ///
    /// # Safety
    /// - Pointer must be valid Multiboot2 structure
    /// - Must be called before any other PMM functions
    pub unsafe fn init(&self, multiboot_info_ptr: usize) {
        use multiboot::Multiboot2Info;
        use multiboot::TagType;

        let info = match Multiboot2Info::new(multiboot_info_ptr) {
            Some(i) => i,
            None => {
                crate::drivers::serial::write_string("Failed to parse multiboot info\n");
                return;
            }
        };

        // Find memory map tag and calculate total available memory
        let mut max_addr: u64 = 0;

        for tag in info.tags() {
            if tag.ty() == TagType::MemoryMap as u16 {
                if let Some(entries) = tag.memory_map_entries() {
                    for entry in entries {
                        // Only count available (type=1) memory regions
                        if entry.ty == multiboot::MEMORY_AVAILABLE {
                            let end = entry.base_addr + entry.length;
                            max_addr = max_addr.max(end);
                        }
                    }
                }
            }
        }

        if max_addr == 0 {
            crate::drivers::serial::write_string("No memory found!\n");
            return;
        }

        // Calculate page counts (4KB pages)
        let total_pages = (max_addr / PAGE_SIZE as u64) as usize;

        // Limit to keep bitmap manageable (can be increased with larger bitmap)
        let total_pages = total_pages.min(MAX_PHYSICAL_PAGES);

        // Calculate bitmap size: 1 bit per page, rounded up to full bytes, then to pages
        let bitmap_bytes = (total_pages + BITS_PER_BYTE - 1) / BITS_PER_BYTE;
        let bitmap_pages = (bitmap_bytes + PAGE_SIZE - 1) / PAGE_SIZE;

        // Ensure bitmap fits in pre-allocated space
        if bitmap_pages > MAX_BITMAP_PAGES {
            crate::drivers::serial::write_string("Bitmap too large!\n");
            return;
        }

        // Get pre-allocated bitmap from BSS section
        #[allow(static_mut_refs)]
        let bitmap_ptr = crate::memory::globals::PMM_BITMAP.as_mut_ptr();

        // Initialize bitmap to zeros (all pages free initially)
        bitmap_ptr.write_bytes(0, bitmap_pages * PAGE_SIZE);

        // Configure PMM
        let pmm_self = self;
        *pmm_self.bitmap.lock() = Bitmap {
            data: bitmap_ptr as usize,
            num_pages: total_pages,
        };
        pmm_self.total_pages.store(total_pages, Ordering::SeqCst);
        pmm_self
            .reserved_pages
            .store(bitmap_pages, Ordering::SeqCst);

        // Mark all pages as free
        for i in 0..total_pages {
            pmm_self.mark_page_free(i);
        }

        // Reserve bitmap storage pages
        for i in 0..bitmap_pages {
            pmm_self.mark_page_used(i);
        }

        // Reserve kernel region (where kernel code is loaded)
        let kernel_start = KERNEL_START / PAGE_SIZE;
        let kernel_pages = KERNEL_SIZE / PAGE_SIZE;
        for i in 0..kernel_pages {
            pmm_self.mark_page_used(kernel_start + i);
        }
    }

    /// Marks a specific page as allocated (used)
    ///
    /// # Arguments
    /// * `page_idx` - Page index (not address)
    #[inline]
    fn mark_page_used(&self, page_idx: usize) {
        let bitmap = self.bitmap.lock();
        let byte_idx = page_idx / BITS_PER_BYTE;
        let bit_idx = page_idx % BITS_PER_BYTE;

        unsafe {
            let ptr = (bitmap.data as *mut u8).add(byte_idx);
            *ptr |= 1 << bit_idx;
        }
    }

    /// Marks a specific page as free (available)
    ///
    /// # Arguments
    /// * `page_idx` - Page index (not address)
    #[inline]
    fn mark_page_free(&self, page_idx: usize) {
        let bitmap = self.bitmap.lock();
        let byte_idx = page_idx / BITS_PER_BYTE;
        let bit_idx = page_idx % BITS_PER_BYTE;

        unsafe {
            let ptr = (bitmap.data as *mut u8).add(byte_idx);
            *ptr &= !(1 << bit_idx);
        }
    }

    /// Checks if a page is currently free (for future use)
    #[allow(dead_code)]
    #[inline]
    fn is_page_free(&self, page_idx: usize) -> bool {
        let bitmap = self.bitmap.lock();
        let byte_idx = page_idx / BITS_PER_BYTE;
        let bit_idx = page_idx % BITS_PER_BYTE;

        unsafe {
            let ptr = (bitmap.data as *mut u8).add(byte_idx);
            (*ptr & (1 << bit_idx)) == 0
        }
    }

    /// Allocates a single physical memory page
    ///
    /// # Returns
    /// - `Some(physical_addr)` - Address of the allocated page (4KB aligned)
    /// - `None` if no pages available
    pub fn allocate_page(&self) -> Option<usize> {
        let bitmap = self.bitmap.lock();

        // Linear search for first free page
        for i in 0..bitmap.num_pages {
            let byte_idx = i / BITS_PER_BYTE;
            let bit_idx = i % BITS_PER_BYTE;

            unsafe {
                let ptr = (bitmap.data as *mut u8).add(byte_idx);
                if (*ptr & (1 << bit_idx)) == 0 {
                    // Found free page - mark as used and return address
                    drop(bitmap);
                    self.mark_page_used(i);
                    return Some(i * PAGE_SIZE);
                }
            }
        }

        None
    }

    /// Allocates multiple contiguous physical memory pages
    ///
    /// Uses first-fit strategy - finds first block of contiguous free pages.
    ///
    /// # Arguments
    /// * `num_pages` - Number of pages to allocate
    ///
    /// # Returns
    /// - `Some(physical_addr)` - Address of first page in block
    /// - `None` if no contiguous block large enough
    pub fn allocate_pages(&self, num_pages: usize) -> Option<usize> {
        let bitmap = self.bitmap.lock();

        let mut consecutive_free = 0;
        let mut start_page = 0;

        for i in 0..bitmap.num_pages {
            let byte_idx = i / BITS_PER_BYTE;
            let bit_idx = i % BITS_PER_BYTE;

            unsafe {
                let ptr = (bitmap.data as *mut u8).add(byte_idx);
                if (*ptr & (1 << bit_idx)) == 0 {
                    // This page is free
                    if consecutive_free == 0 {
                        start_page = i;
                    }
                    consecutive_free += 1;

                    // Check if we have enough contiguous pages
                    if consecutive_free >= num_pages {
                        drop(bitmap);
                        for j in 0..num_pages {
                            self.mark_page_used(start_page + j);
                        }
                        return Some(start_page * PAGE_SIZE);
                    }
                } else {
                    // Page is used - reset counter
                    consecutive_free = 0;
                }
            }
        }

        None
    }

    /// Frees a previously allocated single page
    ///
    /// # Arguments
    /// * `addr` - Physical address of the page to free (must be 4KB aligned)
    pub fn deallocate_page(&self, addr: usize) {
        let page_idx = addr / PAGE_SIZE;
        self.mark_page_free(page_idx);
    }

    /// Frees previously allocated contiguous pages
    ///
    /// # Arguments
    /// * `addr` - Physical address of first page
    /// * `num_pages` - Number of pages to free
    pub fn deallocate_pages(&self, addr: usize, num_pages: usize) {
        let start_idx = addr / PAGE_SIZE;
        for i in 0..num_pages {
            self.mark_page_free(start_idx + i);
        }
    }

    /// Returns the total number of physical pages detected
    pub fn get_total_pages(&self) -> usize {
        self.total_pages.load(Ordering::SeqCst)
    }

    /// Returns the number of currently free pages
    pub fn get_free_pages(&self) -> usize {
        let bitmap = self.bitmap.lock();
        let mut free = 0;

        for i in 0..bitmap.num_pages {
            let byte_idx = i / BITS_PER_BYTE;
            let bit_idx = i % BITS_PER_BYTE;

            unsafe {
                let ptr = (bitmap.data as *mut u8).add(byte_idx);
                if (*ptr & (1 << bit_idx)) == 0 {
                    free += 1;
                }
            }
        }

        free
    }
}

/// Global PMM instance
///
/// Initialized during kernel boot via `PMM.init(multiboot_ptr)`.
/// Use this for all physical memory allocation.
pub static PMM: PhysicalMemoryManager = PhysicalMemoryManager::new();
