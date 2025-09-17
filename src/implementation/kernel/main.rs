#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod panic;
mod std_lib;
mod interrupts;


#[no_mangle] // prevents Rust from mangling the name
pub extern "C" fn kernel_main() -> ! {
    // Kernel code starts here
    std_lib::vga::clear_screen();
    std_lib::vga::println("Hello, World!");
    std_lib::vga::println("This is my kernel.");
}

