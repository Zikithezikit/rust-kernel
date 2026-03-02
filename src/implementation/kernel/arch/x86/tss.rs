//! x86_64 TSS (Task State Segment) implementation
//!
//! Provides the TSS for handling double faults and other CPU exceptions
//! that require a known stack location.

use crate::error::{KernelError, KernelResult};
use crate::memory::pmm::PMM;
use spin::Once;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// Index in the Interrupt Stack Table for the double-fault handler
pub const DOUBLE_FAULT_IST_INDEX: u8 = 0;

/// Global Descriptor Table - lazily initialized
static GDT: Once<GlobalDescriptorTable> = Once::new();

/// Task State Segment - lazily initialized
static TSS: Once<TaskStateSegment> = Once::new();

/// Returns a reference to the GDT if initialized.
pub fn get_gdt() -> Option<&'static GlobalDescriptorTable> {
    GDT.get()
}

/// Returns a reference to the TSS if initialized.
pub fn get_tss() -> Option<&'static TaskStateSegment> {
    TSS.get()
}

/// Initializes the TSS (Task State Segment).
///
/// This function:
/// 1. Allocates a stack page for the double-fault handler
/// 2. Creates a TSS with the double-fault stack
/// 3. Creates and loads the GDT with the TSS descriptor
///
/// # Errors
/// Returns `KernelError::TssInitFailed` if:
/// - Failed to allocate stack memory
/// 
/// # Panics
/// Will panic if either or fails
/// - GDT or TSS initialization / appending to the GDT
pub fn init() -> KernelResult<()> {
    let stack_addr = PMM
        .allocate_pages(1)
        .ok_or(KernelError::StackAllocationFailed)?;

    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = VirtAddr::new(stack_addr as u64);

    let tss_inner = TSS.call_once(|| tss);

    let mut gdt = GlobalDescriptorTable::new();
    gdt.append(Descriptor::kernel_code_segment());
    gdt.append(Descriptor::tss_segment(tss_inner));

    GDT.call_once(|| gdt);

    Ok(())
}

/// Loads the GDT and TSS into the CPU.
///
/// # Errors
/// Returns `KernelError::TssInitFailed` if GDT or TSS not initialized.
pub fn load() -> KernelResult<()> {
    let _gdt = GDT.get().ok_or(KernelError::TssInitFailed)?;
    let _tss = TSS.get().ok_or(KernelError::TssInitFailed)?;

    let tss_selector = SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0);

    unsafe {
        GDT.get().expect("GDT not initialized").load();
        x86_64::instructions::tables::load_tss(tss_selector);
    }

    Ok(())
}
