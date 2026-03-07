//! Fork syscall tests

use crate::arch::x86::userspace::jump_to_user_mode;
use crate::drivers::serial;
use crate::mm::page_tables::map_user_page;
use crate::mm::pmm::PMM;
use x86_64::structures::paging::{Page, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

pub fn test_fork_user() {
    serial::write_string("Testing fork in user mode...\n");

    let code_phys = match PMM.allocate_page() {
        Some(p) => p,
        None => {
            serial::write_string("Failed to allocate code page\n");
            return;
        }
    };
    let stack_phys = match PMM.allocate_page() {
        Some(p) => p,
        None => {
            serial::write_string("Failed to allocate stack page\n");
            return;
        }
    };

    serial::write_string("Allocated phys pages: ");
    serial::write_hex(code_phys as u64);
    serial::write_string(", ");
    serial::write_hex(stack_phys as u64);
    serial::write_string("\n");

    let user_code_virt = 0x1_0000_0000u64; // 4GB
    let user_stack_virt = 0x1_1000_0000u64; // 4.25GB

    // Create a new page table for the user process
    let user_cr3 = crate::mm::page_tables::create_process_page_table()
        .expect("Failed to create user page table");

    unsafe {
        // Activate the new page table so map_user_page can modify it
        crate::mm::page_tables::activate(user_cr3);

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

    // User program with fixed offsets
    let program: [u8; 85] = [
        0xb8, 0x07, 0x00, 0x00, 0x00, // 0: mov eax, 7
        0xcd, 0x80, // 5: int 0x80
        0x83, 0xf8, 0x00, // 7: cmp eax, 0
        0x74, 0x1d, // 10: je .child
        // .parent (starts at 12)
        0xb8, 0x01, 0x00, 0x00, 0x00, // 12: mov eax, 1
        0xbf, 0x01, 0x00, 0x00, 0x00, // 17: mov edi, 1
        0x48, 0xbe, 0x48, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, // 22: mov rsi, 0x100000048
        0xba, 0x07, 0x00, 0x00, 0x00, // 32: mov edx, 7
        0xcd, 0x80, // 37: int 0x80
        0xeb, 0x1b, // 39: jmp .end
        // .child (starts at 41)
        0xb8, 0x01, 0x00, 0x00, 0x00, // 41: mov eax, 1
        0xbf, 0x01, 0x00, 0x00, 0x00, // 46: mov edi, 1
        0x48, 0xbe, 0x4f, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, // 51: mov rsi, 0x10000004f
        0xba, 0x06, 0x00, 0x00, 0x00, // 61: mov edx, 6
        0xcd, 0x80, // 66: int 0x80
        // .end (starts at 68)
        0xeb, 0xfe, // 68: jmp $
        // 70: padding
        0x00, 0x00, // 72 (0x48): 'PARENT\n'
        b'P', b'A', b'R', b'E', b'N', b'T', b'\n', // 79 (0x4F): 'CHILD\n'
        b'C', b'H', b'I', b'L', b'D', b'\n',
    ];

    unsafe {
        let code_ptr = code_phys as *mut u8;
        core::ptr::copy_nonoverlapping(program.as_ptr(), code_ptr, program.len());
    }

    serial::write_string("Jumping to user mode for fork test...\n");

    unsafe {
        // We need a kernel stack for the child
        let kernel_stack = PMM
            .allocate_page()
            .expect("Failed to allocate kernel stack");
        crate::arch::x86::tss::set_kernel_stack(VirtAddr::new((kernel_stack + 4096) as u64));

        x86_64::instructions::interrupts::disable();
        jump_to_user_mode(user_code_virt, user_stack_virt + 4096);
    }
}
