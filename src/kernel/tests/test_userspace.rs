//! User mode transition tests

use crate::arch::x86::userspace::jump_to_user_mode;
use crate::drivers::serial;
use crate::mm::page_tables::map_user_page;
use crate::mm::pmm::PMM;
use x86_64::structures::paging::{Page, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

pub fn test_user_mode_transition() {
    serial::write_string("Testing user mode transition...\n");

    // 1. Create a new page table for the user process
    let user_cr3 = crate::mm::page_tables::create_process_page_table()
        .expect("Failed to create user page table");

    // 2. Allocate code and stack pages
    let code_phys = PMM.allocate_page().expect("Failed to allocate code page");
    let stack_phys = PMM.allocate_page().expect("Failed to allocate stack page");

    // 3. Activate the new page table so we can map into it
    unsafe {
        crate::mm::page_tables::activate(user_cr3);
    }

    // 4. Map them for user access
    // We'll map them at specific virtual addresses to avoid overlap with kernel (first 1GB)
    let user_code_virt = 0x4000_0000u64; // 1GB
    let user_stack_virt = 0x4000_1000u64; // 1GB + 4KB

    unsafe {
        map_user_page(
            Page::containing_address(VirtAddr::new(user_code_virt)),
            PhysFrame::containing_address(PhysAddr::new(code_phys as u64)),
        )
        .expect("Failed to map user code page");

        map_user_page(
            Page::containing_address(VirtAddr::new(user_stack_virt)),
            PhysFrame::containing_address(PhysAddr::new(stack_phys as u64)),
        )
        .expect("Failed to map user stack page");
    }

    // 5. Copy a simple user program
    // mov eax, 5 (getpid)
    // int 0x80
    // jmp $
    let program: [u8; 9] = [
        0xb8, 0x05, 0x00, 0x00, 0x00, // mov eax, 5
        0xcd, 0x80, // int 0x80
        0xeb, 0xfe, // jmp -2
    ];

    unsafe {
        let code_ptr = code_phys as *mut u8;
        core::ptr::copy_nonoverlapping(program.as_ptr(), code_ptr, program.len());
    }

    serial::write_string("Jumping to user mode... (Expect syscall 5)\n");

    unsafe {
        // Use the current task's kernel stack for user mode transition
        if let Some(task) = crate::kernel::scheduler::SCHEDULER.current_task() {
            let stack_top = task.lock().kernel_stack_top;
            crate::arch::x86::tss::set_kernel_stack(VirtAddr::new(stack_top as u64));
        }

        // Disable interrupts before jump to ensure stability
        // (iretq will re-enable them because of rflags 0x202)
        x86_64::instructions::interrupts::disable();
        jump_to_user_mode(user_code_virt, user_stack_virt + 4096);
    }
}
