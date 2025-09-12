// src/implementation/kernel/panic.rs
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
