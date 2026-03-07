//! Context switching FFI
//!
//! Provides Rust bindings for the assembly context switch routines.

extern "C" {
    /// Switch to a new task
    ///
    /// # Arguments
    /// * `new_task_rsp` - The kernel stack pointer of the new task
    ///
    /// # Safety
    /// - This function switches the stack and never returns to the caller
    ///   if a valid task is running. Instead, it returns to the new task.
    pub fn context_switch(new_task_rsp: usize);
}

/// Pointer to the current task's Task struct
/// This is used by the assembly context_switch to save/restore state
#[no_mangle]
#[link_section = ".bss"]
pub static mut current_task_ptr: usize = 0;

use super::task::Task;

/// Switches to the given task
///
/// # Safety
/// - Must only be called with a valid task that has a properly set up stack
pub unsafe fn switch_to_task(task: &Task) {
    // 1. Update CR3 if necessary
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::PhysFrame;
    use x86_64::{PhysAddr, VirtAddr};

    let (current_cr3_frame, cr3_flags) = Cr3::read();
    if current_cr3_frame.start_address().as_u64() != task.cr3 as u64 {
        Cr3::write(
            PhysFrame::containing_address(PhysAddr::new(task.cr3 as u64)),
            cr3_flags,
        );
    }

    // 2. Update TSS RSP0
    crate::arch::x86::tss::set_kernel_stack(VirtAddr::new(task.kernel_stack_top as u64));

    // 3. Switch context
    let rsp = task.kernel_stack;
    context_switch(rsp);
}
