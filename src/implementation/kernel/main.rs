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

use drivers::serial;
use memory::allocator::PmmAllocator;

#[global_allocator]
static ALLOCATOR: PmmAllocator = PmmAllocator::new();

#[inline(always)]
fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[inline(always)]
fn log_on_err<E: core::fmt::Display>(result: Result<(), E>, msg: &str) {
    if let Err(e) = result {
        serial::write_string(&alloc::format!("{}: {}\n", msg, e));
    }
}

#[inline(always)]
fn halt_on_err<E: core::fmt::Display>(result: Result<(), E>, msg: &str) {
    if let Err(e) = result {
        serial::write_string(&alloc::format!("{}: {}\n", msg, e));
        halt();
    }
}

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: usize) -> ! {
    drivers::init::init();
    halt_on_err(
        memory::init::init_memory(multiboot_info, &ALLOCATOR),
        "Memory init failed",
    );

    drivers::vga::clear_screen();
    drivers::vga::println("Hello, World!");
    drivers::vga::println("This is my kernel.");
    serial::write_string("Serial initialized!\n");

    halt_on_err(arch::init::init_tss(), "TSS init failed");

    tests::run_tests();

    log_on_err(arch::init::load_tss(), "TSS load failed");
    halt_on_err(arch::init::init_interrupts(), "Interrupts init failed");
    halt_on_err(task::init::init(), "Task init failed");

    halt();
}
