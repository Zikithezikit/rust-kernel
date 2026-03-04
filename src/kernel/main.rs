#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod arch;
mod drivers;
mod fs;
mod include;
mod ipc;
mod kernel;
mod kernel_instance;
mod memory;
mod mm;
mod panic;
mod tests;

use drivers::serial;
use mm::allocator::PmmAllocator;

#[global_allocator]
static ALLOCATOR: PmmAllocator = PmmAllocator::new();

#[inline(always)]
fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[inline(always)]
fn halt_on_err<E: core::fmt::Display>(result: Result<(), E>, msg: &str) {
    if let Err(e) = result {
        serial::write_string(&alloc::format!("{}: {}\n", msg, e));
        halt();
    }
}

/// Init sub modules, this function can't fail so there's no return
#[inline(always)]
fn init_submodules(multiboot_info: usize) {
    // Initialize drivers first (VGA, serial)
    halt_on_err(drivers::init::init(), "Driver init failed");
    serial::write_string("Serial initialized!\n");

    // Initialize memory subsystem
    halt_on_err(
        memory::init::init_memory(multiboot_info, &ALLOCATOR),
        "Memory init failed",
    );

    // Initialize the global kernel instance
    halt_on_err(kernel_instance::init_kernel(), "Kernel init failed");

    // Initialize Arch
    halt_on_err(arch::init::init_arch(), "Arch init failed");

    // Init VFS
    halt_on_err(fs::vfs::init(), "VFS init failed");
}

/// This is the main function that is called from the assembly
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: usize) -> ! {
    init_submodules(multiboot_info);

    drivers::vga::clear_screen();
    drivers::vga::println("Hello, World!");
    drivers::vga::println("This is my kernel.");

    tests::run_tests();

    // Run the scheduler - this is the main kernel loop
    serial::write_string("Starting scheduler...\n");
    kernel_instance::kernel().scheduler_mut().run();
}
