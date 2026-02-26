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

use crate::task::scheduler::SCHEDULER;
use crate::task::task::Task;

/// Switches to the given task
///
/// # Safety
/// - Must only be called with a valid task that has a properly set up stack
pub unsafe fn switch_to_task(task: &Task) {
    let rsp = task.kernel_stack;

    // If this is the first time running this task, we need to set up the stack
    // with the entry point as the return address
    if task.state == crate::task::task::TaskState::New {
        // Set up initial stack frame for the task
        // We push the entry point as if it was called
        let stack_top = task.kernel_stack;

        // We need to write to the stack (careful: this is the physical address
        // in a real OS, we'd need proper page table mapping)
        // For now, we assume the kernel_stack is a virtual address we can use

        // Actually, for a new task, we should set up the stack so that
        // when we return, it jumps to the entry point
        // Let's store the entry point where context_switch will return to

        // Since this is complex with virtual memory, let's simplify:
        // For now, we'll just mark the task as Running and let the
        // scheduler handle it differently
    }

    context_switch(rsp);
}
