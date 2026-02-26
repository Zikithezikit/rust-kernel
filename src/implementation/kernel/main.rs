#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod allocator;
mod drivers;
mod globals;
mod interrupts;
mod memory;
mod panic;
mod tests;

use allocator::PmmAllocator;
use memory::init::init_memory;
use x86_64::instructions::interrupts as x86_64_interrupts;

#[global_allocator]
static ALLOCATOR: PmmAllocator = PmmAllocator::new();

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: usize) -> ! {
    drivers::vga::clear_screen();
    drivers::vga::println("Starting...");

    unsafe {
        drivers::serial::init();
    }
    drivers::serial::write_string("Kernel started\n");

    if !init_memory(multiboot_info, &ALLOCATOR) {
        drivers::serial::write_string("Memory init failed! Halting.\n");
        loop {
            x86_64::instructions::hlt();
        }
    }

    drivers::vga::clear_screen();
    drivers::vga::println("Hello, World!");
    drivers::vga::println("This is my kernel.");

    drivers::serial::write_string("Serial initialized!\n");

    tests::run_tests();

    unsafe {
        interrupts::init_pic();
    }
    interrupts::init_idt();
    x86_64_interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}
