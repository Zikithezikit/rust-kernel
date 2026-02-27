use crate::memory::pmm::PMM;
use spin::Once;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u8 = 0;

static GDT: Once<GlobalDescriptorTable> = Once::new();
static TSS: Once<TaskStateSegment> = Once::new();

pub fn get_gdt() -> Option<&'static GlobalDescriptorTable> {
    GDT.get()
}

pub fn get_tss() -> Option<&'static TaskStateSegment> {
    TSS.get()
}

pub fn init() {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        let stack = PMM.allocate_pages(1).expect("Failed to allocate TSS stack");
        VirtAddr::new(stack as u64)
    };
    TSS.call_once(|| tss);

    let mut gdt = GlobalDescriptorTable::new();
    gdt.append(Descriptor::kernel_code_segment());
    gdt.append(Descriptor::tss_segment(TSS.get().unwrap()));

    GDT.call_once(|| gdt);
}

pub fn load() {
    let gdt = GDT.get().expect("GDT not initialized");
    let _tss = TSS.get().expect("TSS not initialized");

    let tss_selector = SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0);

    unsafe {
        gdt.load();
        x86_64::instructions::tables::load_tss(tss_selector);
    }
}
