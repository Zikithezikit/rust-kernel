//! Pre-allocated memory for early kernel use
///
/// These are globals that need to be allocated at link time (BSS section)
/// before we have a working allocator.

const PAGE_SIZE: usize = 4096;
const BITMAP_PAGES: usize = 64;

/// Pre-allocated bitmap for PMM
/// Each bit represents one 4KB physical page
/// Total: 64 * 4096 = 256KB, covering up to 4GB of physical memory
#[no_mangle]
#[link_section = ".bss"]
pub static mut PMM_BITMAP: [u8; PAGE_SIZE * BITMAP_PAGES] = [0; PAGE_SIZE * BITMAP_PAGES];
