#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod arch;
mod drivers;
mod memory;
mod panic;
mod syscall;
mod task;
mod tests;

use alloc::format;
use drivers::serial;
use memory::allocator::PmmAllocator;
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

    if let Err(e) = arch::x86::tss::init() {
        serial::write_string(&format!("TSS init failed: {:?}\n", e));
    }

    tests::run_tests();

    if let Err(e) = arch::x86::tss::load() {
        serial::write_string(&format!("TSS load failed: {:?}\n", e));
    }

    unsafe {
        arch::x86::interrupts::init_pic();
        arch::x86::interrupts::init_pit();
    }
    arch::x86::interrupts::init_idt();
    x86_64_interrupts::enable();

    task::scheduler::SCHEDULER.init();

    fn idle_task() {
        loop {
            x86_64::instructions::hlt();
        }
    }
    task::scheduler::SCHEDULER.spawn(idle_task, "idle");

    loop {
        x86_64::instructions::hlt();
    }
}
