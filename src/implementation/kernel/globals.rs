//! Pre-allocated memory for early kernel use
///
/// These are globals that need to be allocated at link time (BSS section)
/// before we have a working allocator.

/// Pre-allocated bitmap for PMM (64 pages = 256KB)
/// Each bit represents one 4KB physical page
#[no_mangle]
#[link_section = ".bss"]
pub static mut PMM_BITMAP: [u8; 4096 * 64] = [0; 4096 * 64];
