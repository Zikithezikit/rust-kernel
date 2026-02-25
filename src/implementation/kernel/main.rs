#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupts;
mod panic;
mod std_lib;

use x86_64::instructions::interrupts as x86_64_interrupts;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    std_lib::vga::clear_screen();
    std_lib::vga::println("Hello, World!");
    std_lib::vga::println("This is my kernel.");

    unsafe {
        std_lib::serial::init();
    }
    std_lib::serial::write_string("Serial initialized!\n");

    unsafe {
        interrupts::init_pic();
    }
    interrupts::init_idt();
    x86_64_interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}
