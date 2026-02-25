#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod allocator;
mod globals;
mod init;
mod interrupts;
mod panic;
mod pmm;
mod std_lib;
mod tests;
mod vmm;

use allocator::PmmAllocator;
use init::init_memory;
use x86_64::instructions::interrupts as x86_64_interrupts;

/// Global allocator for Rust's heap allocations (Vec, Box, String, etc.)
#[global_allocator]
static ALLOCATOR: PmmAllocator = PmmAllocator::new();

/// Main kernel entry point
///
/// Called from 64-bit assembly entry after paging is enabled.
/// Receives Multiboot2 info pointer in RDI (first argument).
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: usize) -> ! {
    // Clear screen and show initial message
    std_lib::vga::clear_screen();
    std_lib::vga::println("Starting...");

    // Initialize serial port for logging
    unsafe {
        std_lib::serial::init();
    }
    std_lib::serial::write_string("Kernel started\n");

    // Initialize memory management (PMM + heap)
    if !init_memory(multiboot_info, &ALLOCATOR) {
        std_lib::serial::write_string("Memory init failed! Halting.\n");
        loop {
            x86_64::instructions::hlt();
        }
    }

    // Display welcome message
    std_lib::vga::clear_screen();
    std_lib::vga::println("Hello, World!");
    std_lib::vga::println("This is my kernel.");

    std_lib::serial::write_string("Serial initialized!\n");

    // Run built-in tests
    tests::run_tests();

    // Initialize interrupt handling
    unsafe {
        interrupts::init_pic();
    }
    interrupts::init_idt();
    x86_64_interrupts::enable();

    // Main idle loop - kernel is now running
    loop {
        x86_64::instructions::hlt();
    }
}
