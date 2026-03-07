//! Userspace transition logic
//!
//! Handles the transition from kernel mode (Ring 0) to user mode (Ring 3).

use crate::arch::x86::tss::get_selectors;

/// Jump to user mode
///
/// This function transitions the CPU to Ring 3 and starts executing
/// at the given entry point with the provided stack pointer.
///
/// # Safety
/// - The entry point must be a valid user-mode address.
/// - The stack pointer must point to a valid user-mode stack.
/// - The page tables must be set up to allow user-mode access to these addresses.
pub unsafe fn jump_to_user_mode(entry_point: u64, stack_pointer: u64) -> ! {
    let selectors = get_selectors().expect("GDT not initialized");

    let ss = (selectors.user_data.0 | 3) as u64; // Set RPL to 3
    let cs = (selectors.user_code.0 | 3) as u64; // Set RPL to 3
    let rflags = 0x202; // Interrupts enabled, bit 1 always set

    core::arch::asm!(
        "mov ds, {ds:x}",
        "mov es, {ds:x}",
        "push {ss:r}",
        "push {rsp:r}",
        "push {rflags:r}",
        "push {cs:r}",
        "push {rip:r}",
        "iretq",
        ds = in(reg) (selectors.user_data.0 | 3),
        ss = in(reg) ss,
        rsp = in(reg) stack_pointer,
        rflags = in(reg) rflags,
        cs = in(reg) cs,
        rip = in(reg) entry_point,
        options(noreturn)
    );
}
