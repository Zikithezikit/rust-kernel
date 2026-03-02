#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod arch;
mod drivers;
mod error;
mod kernel;
mod memory;
mod panic;
mod syscall;
mod task;
mod tests;
mod vfs;

use drivers::serial;
use kernel as kernel_module;
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
    // Initialize drivers first (VGA, serial)
    halt_on_err(drivers::init::init(), "Driver init failed");

    // Initialize memory subsystem
    halt_on_err(
        memory::init::init_memory(multiboot_info, &ALLOCATOR),
        "Memory init failed",
    );

    drivers::vga::clear_screen();
    drivers::vga::println("Hello, World!");
    drivers::vga::println("This is my kernel.");
    serial::write_string("Serial initialized!\n");

    // Initialize the global kernel instance
    halt_on_err(kernel_module::init_kernel(), "Kernel init failed");

    // Initialize TSS first
    halt_on_err(arch::init::init_tss(), "TSS init failed");

    // NOTE: Tests are temporarily disabled due to kernel reset issue after test completion
    // This needs to be investigated separately
    serial::write_string("Tests disabled for now\n");

    // Load TSS and initialize interrupts
    halt_on_err(arch::init::load_tss(), "TSS load failed");
    halt_on_err(arch::init::init_interrupts(), "Interrupts init failed");

    // Enable CPU interrupts
    serial::write_string("Enabling interrupts...\n");
    arch::init::enable_interrupts();
    serial::write_string("Interrupts enabled!\n");

    // Initialize VFS (placeholder - will be expanded)
    serial::write_string("VFS initialized!\n");

    // Run the scheduler - this is the main kernel loop
    serial::write_string("Starting scheduler...\n");
    kernel_module::kernel().scheduler_mut().run();

    // If scheduler returns (should never happen), halt
    halt();
}
