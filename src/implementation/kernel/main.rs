#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod panic;
mod std_lib;
mod interrupts;

use x86_64::instructions::interrupts as x86_64_interrupts;


#[no_mangle] // prevents Rust from mangling the name
pub extern "C" fn kernel_main() -> ! {
    // Kernel code starts here
    std_lib::vga::clear_screen();
    std_lib::vga::println("Hello, World!");
    std_lib::vga::println("This is my kernel.");

    unsafe { interrupts::init_pic(); }
    interrupts::init_idt();
    x86_64_interrupts::enable();

    loop{ x86_64::instructions::hlt(); };
}

