#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod allocator;
mod interrupts;
mod panic;
mod std_lib;
mod tests;
mod vmm;

use allocator::BumpAllocator;
use x86_64::instructions::interrupts as x86_64_interrupts;

const HEAP_START: usize = 0x_100_000;
const HEAP_SIZE: usize = 0x_10_000;

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    ALLOCATOR.init();

    std_lib::vga::clear_screen();
    std_lib::vga::println("Hello, World!");
    std_lib::vga::println("This is my kernel.");

    unsafe {
        std_lib::serial::init();
    }
    std_lib::serial::write_string("Serial initialized!\n");

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
