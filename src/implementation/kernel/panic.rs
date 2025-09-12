// src/implementation/kernel/panic.rs

use core::panic::PanicInfo;
use std_lib;

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    std_lib::vga::clear_screen();
    std_lib::vga::println("Kernel Panic!");
    std_lib::vga::println("System halted.");
    
    loop {}
}
