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
        l4_table_ptr.write(PageTable::new());
    }
    let l4_table = unsafe { &mut *l4_table_ptr };

    // 2. Allocate a page for the L3 table (to allow user mappings while sharing kernel)
    let l3_phys = PMM.allocate_page()?;
    let l3_table_ptr = l3_phys as *mut PageTable;
    unsafe {
        l3_table_ptr.write(PageTable::new());
    }
    let l3_table = unsafe { &mut *l3_table_ptr };

    unsafe {
        // 3. Get the active L4 and L3 tables
        let (active_l4_frame, _) = Cr3::read();
        let active_l4 = &*(active_l4_frame.start_address().as_u64() as *const PageTable);

        let active_l3_entry = &active_l4[0];
        if !active_l3_entry.is_unused() {
            let active_l3 = &*(active_l3_entry.addr().as_u64() as *const PageTable);

            // 4. Share the kernel mapping (first 4GB)
            // In our updated boot loader, L3[0..3] cover the first 4GB
            for i in 0..4 {
                l3_table[i] = active_l3[i].clone();
            }

            // 5. Connect L4[0] to our new L3
            l4_table[0].set_addr(
                PhysAddr::new(l3_phys as u64),
                active_l3_entry.flags() | PageTableFlags::USER_ACCESSIBLE,
            );
        }

        // Copy other potential kernel mappings (e.g. higher half if we had any)
        for i in 1..512 {
            if !active_l4[i].is_unused()
                && !active_l4[i]
                    .flags()
                    .contains(PageTableFlags::USER_ACCESSIBLE)
            {
                l4_table[i] = active_l4[i].clone();
            }
        }
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

/// Deep copy an address space (used for fork)
pub unsafe fn copy_address_space(src_cr3: PhysAddr) -> Option<PhysAddr> {
    let dst_cr3 = create_process_page_table()?;

    let src_l4 = &*(src_cr3.as_u64() as *const PageTable);
    let dst_l4 = &mut *(dst_cr3.as_u64() as *mut PageTable);

    // Copy L4 entries
    for i in 0..512 {
        let src_entry = &src_l4[i];
        if src_entry.is_unused() {
            continue;
        }

        // We only copy user-accessible entries
        if src_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE) {
            if i == 0 {
                // Entry 0 is special (contains kernel in first 1GB)
                // create_process_page_table already created a new L3 for us
                let src_l3_phys = src_entry.addr().as_u64() as usize;
                let dst_l3_phys = dst_l4[0].addr().as_u64() as usize;
                copy_table_contents(src_l3_phys, dst_l3_phys, 3);
            } else {
                // General user entries (above 512GB, unlikely for now but good for completeness)
                if let Some(dst_subtree_phys) = copy_subtree(src_entry.addr().as_u64() as usize, 3)
                {
                    dst_l4[i].set_addr(PhysAddr::new(dst_subtree_phys as u64), src_entry.flags());
                }
            }
        }
    }

    Some(dst_cr3)
}

unsafe fn copy_subtree(src_table_phys: usize, level: u8) -> Option<usize> {
    let dst_table_phys = PMM.allocate_page()?;
    let dst_table_ptr = dst_table_phys as *mut PageTable;
    dst_table_ptr.write(PageTable::new());

    copy_table_contents(src_table_phys, dst_table_phys, level);
    Some(dst_table_phys)
}

unsafe fn copy_table_contents(src_table_phys: usize, dst_table_phys: usize, level: u8) {
    let src_table = &*(src_table_phys as *const PageTable);
    let dst_table = &mut *(dst_table_phys as *mut PageTable);

    // In L3, index 0..3 are kernel space (first 4GB), we skip them as they were already
    // shared by create_process_page_table
    let start_index = if level == 3 { 4 } else { 0 };

    for i in start_index..512 {
        let entry = &src_table[i];
        if entry.is_unused() {
            continue;
        }

        // Only copy user-accessible entries
        if entry.flags().contains(PageTableFlags::USER_ACCESSIBLE) {
            if level > 1 && !entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                // Internal node, deep copy subtree
                if let Some(dst_next_phys) = copy_subtree(entry.addr().as_u64() as usize, level - 1)
                {
                    dst_table[i].set_addr(PhysAddr::new(dst_next_phys as u64), entry.flags());
                }
            } else {
                // Leaf node (L1 or huge page), copy data
                if let Some(dst_data_phys) = PMM.allocate_page() {
                    // NOTE: Assumes physical addresses are accessible (identity mapped)
                    core::ptr::copy_nonoverlapping(
                        entry.addr().as_u64() as *const u8,
                        dst_data_phys as *mut u8,
                        4096,
                    );
                    dst_table[i].set_addr(PhysAddr::new(dst_data_phys as u64), entry.flags());
                }
            }
        }
    }
}
