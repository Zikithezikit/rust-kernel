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

    // Load TSS and initialize interrupts FIRST
    halt_on_err(arch::init::load_tss(), "TSS load failed");
    halt_on_err(arch::init::init_interrupts(), "Interrupts init failed");

    // Enable CPU interrupts
    serial::write_string("Enabling interrupts...\n");
    arch::init::enable_interrupts();
    serial::write_string("Interrupts enabled!\n");

    // Run kernel tests AFTER interrupts are enabled
    serial::write_string("=== Running kernel tests ===\n");

    // Basic tests
    tests::test_allocator::test_vec_allocation();
    tests::test_allocator::test_box_allocation();
    tests::test_allocator::test_string_allocation();
    tests::test_allocator::test_multiple_allocations();
    tests::test_panic::test_panic_constants();
    tests::test_panic::test_serial_hex_output();
    tests::test_pmm::test_pmm_initialized();
    tests::test_pmm::test_pmm_allocate_single_page();
    tests::test_pmm::test_pmm_allocate_multiple_pages();
    tests::test_pmm::test_pmm_stress();

    // VFS tests
    tests::test_vfs::test_vfs_inode_creation();
    tests::test_vfs::test_vfs_directory_inode();
    tests::test_vfs::test_vfs_inode_mkdir();
    tests::test_vfs::test_vfs_inode_create_file();
    tests::test_vfs::test_vfs_inode_read_write();

    serial::write_string("=== Tests completed ===\n");

    // Initialize VFS
    serial::write_string("VFS initialized!\n");

    // Run the scheduler - this is the main kernel loop
    serial::write_string("Starting scheduler...\n");
    kernel_module::kernel().scheduler_mut().run();

    // If scheduler returns (should never happen), halt
    halt();
}
