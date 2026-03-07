//! Page table management
//!
//! Provides abstractions for managing x86_64 page tables.

use crate::mm::pmm::PMM;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

/// Frame allocator that uses the kernel's Physical Memory Manager (PMM)
pub struct KernelFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for KernelFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        PMM.allocate_page()
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr as u64)))
    }
}

/// Initialize a new page table for a process
///
/// Returns the physical address of the level 4 page table.
pub fn create_process_page_table() -> Option<PhysAddr> {
    // 1. Allocate a page for the L4 table
    let l4_phys = PMM.allocate_page()?;
    let l4_table_ptr = l4_phys as *mut PageTable;

    unsafe {
        // 2. Clear the new table
        l4_table_ptr.write(PageTable::new());
        let l4_table = &mut *l4_table_ptr;

        // 3. Copy kernel mappings from the active L4 table
        let (active_l4_frame, _) = Cr3::read();
        let active_l4_ptr = active_l4_frame.start_address().as_u64() as *const PageTable;
        let active_l4 = &*active_l4_ptr;

        // Copy the first entry (which contains our identity mapping of first 1GB)
        // We also set the USER_ACCESSIBLE bit on this L4 entry so that
        // user-mode mappings can be created within this 512GB range.
        // The actual protection for kernel pages is still enforced by L3, L2, and L1 entries
        // which do NOT have the USER_ACCESSIBLE bit set.
        let mut entry = active_l4[0].clone();
        if !entry.is_unused() {
            entry.set_flags(entry.flags() | PageTableFlags::USER_ACCESSIBLE);
        }
        l4_table[0] = entry;

        // In the future, we might want to copy more entries if the kernel is mapped elsewhere
    }

    Some(PhysAddr::new(l4_phys as u64))
}

/// Map a virtual page to a physical frame in the current address space
pub unsafe fn map_page(
    page: Page<Size4KiB>,
    frame: PhysFrame,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    let mut mapper = get_active_mapper();
    let mut frame_allocator = KernelFrameAllocator;

    match mapper.map_to(page, frame, flags, &mut frame_allocator) {
        Ok(tlb) => {
            tlb.flush();
            Ok(())
        }
        Err(_) => Err("Failed to map page"),
    }
}

/// Map a virtual page to a physical frame with user-mode access
pub unsafe fn map_user_page(page: Page<Size4KiB>, frame: PhysFrame) -> Result<(), &'static str> {
    let flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
    map_page(page, frame, flags)
}

/// Get a mapper for the active page table
///
/// SAFETY: This assumes identity mapping for page table access.
pub unsafe fn get_active_mapper() -> OffsetPageTable<'static> {
    let (l4_frame, _) = Cr3::read();
    let l4_ptr = l4_frame.start_address().as_u64() as *mut PageTable;
    let l4_table = &mut *l4_ptr;

    // Since we use identity mapping, the offset is 0
    OffsetPageTable::new(l4_table, VirtAddr::new(0))
}

/// Activate a page table by loading its physical address into CR3
pub unsafe fn activate(l4_phys: PhysAddr) {
    let (active_frame, flags) = Cr3::read();
    if active_frame.start_address() != l4_phys {
        Cr3::write(PhysFrame::containing_address(l4_phys), flags);
    }
}
