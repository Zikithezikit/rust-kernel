#![no_std]
#![no_main]

mod panic;
mod std_lib;


#[no_mangle] // prevents Rust from mangling the name
pub extern "C" fn kernel_main() -> ! {
    // Kernel code starts here
    std_lib::vga::clear_screen();
    std_lib::vga::println("Hello, World!");
    std_lib::vga::println("This is my kernel.");
    loop {}
}

