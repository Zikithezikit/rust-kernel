use crate::drivers::serial;
use crate::memory::pmm::PMM;
use spin::Once;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u8 = 0;

static GDT: Once<GlobalDescriptorTable> = Once::new();
static TSS: Once<TaskStateSegment> = Once::new();

#[derive(Debug)]
pub enum TssError {
    FailedToAllocateStack,
    GdtNotInitialized,
    TssNotInitialized,
}

impl core::fmt::Display for TssError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TssError::FailedToAllocateStack => write!(f, "FailedToAllocateStack"),
            TssError::GdtNotInitialized => write!(f, "GdtNotInitialized"),
            TssError::TssNotInitialized => write!(f, "TssNotInitialized"),
        }
    }
}

pub fn get_gdt() -> Option<&'static GlobalDescriptorTable> {
    GDT.get()
}

pub fn get_tss() -> Option<&'static TaskStateSegment> {
    TSS.get()
}

pub fn init() -> Result<(), TssError> {
    let stack_addr = PMM
        .allocate_pages(1)
        .ok_or(TssError::FailedToAllocateStack)?;

    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = VirtAddr::new(stack_addr as u64);

    let tss_inner = TSS.call_once(|| tss);

    let mut gdt = GlobalDescriptorTable::new();
    gdt.append(Descriptor::kernel_code_segment());
    gdt.append(Descriptor::tss_segment(tss_inner));

    GDT.call_once(|| gdt);

    Ok(())
}

pub fn load() -> Result<(), TssError> {
    let gdt = GDT.get().ok_or(TssError::GdtNotInitialized)?;
    let _tss = TSS.get().ok_or(TssError::TssNotInitialized)?;

    let tss_selector = SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0);

    unsafe {
        gdt.load();
        x86_64::instructions::tables::load_tss(tss_selector);
    }

    Ok(())
}
