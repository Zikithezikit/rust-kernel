//! x86_64 TSS (Task State Segment) implementation
//!
//! Provides the TSS for handling double faults and other CPU exceptions
//! that require a known stack location.

use crate::include::error::{KernelError, KernelResult};
use crate::mm::pmm::PMM;
use spin::Once;
use x86_64::instructions::segmentation::{Segment, CS, DS, ES, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// Index in the Interrupt Stack Table for the double-fault handler
pub const DOUBLE_FAULT_IST_INDEX: u8 = 0;

/// Global Descriptor Table - lazily initialized
static GDT: Once<(GlobalDescriptorTable, Selectors)> = Once::new();

/// Task State Segment - lazily initialized
static TSS: Once<TaskStateSegment> = Once::new();

pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub tss: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
}

/// Returns a reference to the GDT if initialized.
pub fn get_gdt() -> Option<&'static GlobalDescriptorTable> {
    GDT.get().map(|(gdt, _)| gdt)
}

/// Returns the selectors if initialized.
pub fn get_selectors() -> Option<&'static Selectors> {
    GDT.get().map(|(_, selectors)| selectors)
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
/// 3. Creates and loads the GDT with descriptors for kernel and user mode
///
/// # Errors
/// Returns `KernelError::StackAllocationFailed` if failed to allocate stack memory
///
/// # Panics
/// Will panic if GDT initialization / appending to the GDT fails
pub fn init() -> KernelResult<()> {
    let stack_addr = PMM
        .allocate_pages(1)
        .ok_or(KernelError::StackAllocationFailed)?;

    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = VirtAddr::new(stack_addr as u64);

    let tss_inner = TSS.call_once(|| tss);

    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data = gdt.append(Descriptor::kernel_data_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(tss_inner));
    let user_data = gdt.append(Descriptor::user_data_segment());
    let user_code = gdt.append(Descriptor::user_code_segment());

    GDT.call_once(|| {
        (
            gdt,
            Selectors {
                kernel_code,
                kernel_data,
                tss: tss_selector,
                user_data,
                user_code,
            },
        )
    });

    Ok(())
}

/// Loads the GDT and TSS into the CPU.
///
/// # Errors
/// Returns `KernelError::TssInitFailed` if GDT or TSS not initialized.
pub fn load() -> KernelResult<()> {
    let (gdt, selectors) = GDT.get().ok_or(KernelError::TssInitFailed)?;
    let _tss = TSS.get().ok_or(KernelError::TssInitFailed)?;

    unsafe {
        gdt.load();
        CS::set_reg(selectors.kernel_code);
        DS::set_reg(selectors.kernel_data);
        ES::set_reg(selectors.kernel_data);
        SS::set_reg(selectors.kernel_data);
        x86_64::instructions::tables::load_tss(selectors.tss);
    }

    Ok(())
}

/// Sets the kernel stack for the current CPU in the TSS.
/// This is used during task switching to ensure interrupts in user mode
/// use the correct kernel stack.
pub fn set_kernel_stack(stack_addr: VirtAddr) {
    if let Some(tss) = TSS.get() {
        // SAFETY: We have a reference to the static TSS.
        // In a multi-core system, we would need a TSS per core.
        let tss_ptr = tss as *const TaskStateSegment as *mut TaskStateSegment;
        unsafe {
            (*tss_ptr).privilege_stack_table[0] = stack_addr;
        }
    }
}
