// src/implementation/kernel/panic.rs

use crate::std_lib;
use core::panic::PanicInfo;

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    std_lib::vga::clear_screen();
    std_lib::vga::println("Kernel Panic!");
    std_lib::vga::println("System halted.");

    loop {}
}
