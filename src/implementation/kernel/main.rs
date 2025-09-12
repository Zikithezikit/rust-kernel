#![no_std]
#![no_main]

mod panic;

#[no_mangle] // prevents Rust from mangling the name
pub extern "C" fn kernel_main() -> ! {
    // Kernel code starts here
    loop {}
}

