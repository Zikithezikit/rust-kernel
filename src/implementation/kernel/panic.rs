// src/implementation/kernel/panic.rs
//!
//! Panic handler for the kernel.
//! Outputs panic information to both VGA and serial ports.

use crate::std_lib;
use alloc::format;
use core::panic::PanicInfo;

const PANIC_SEPARATOR: &str = "======================================";
const VGA_PANIC_SEPARATOR: &str = "========================================";
const MSG_PANIC_TITLE: &str = "KERNEL PANIC";
const MSG_SYSTEM_HALTED: &str = "System halted. Check serial for details.";
const MSG_QEMU_EXIT: &str = "QEMU exit: Ctrl+Alt+Del or close window";

fn write_hex(value: u64) {
    std_lib::serial::write_hex(value);
}

fn print_panic_header(info: &PanicInfo) {
    let location = info.location();

    std_lib::serial::write_string(PANIC_SEPARATOR);
    std_lib::serial::write_string("\n");
    std_lib::serial::write_string("           ");
    std_lib::serial::write_string(MSG_PANIC_TITLE);
    std_lib::serial::write_string("              \n");
    std_lib::serial::write_string(PANIC_SEPARATOR);
    std_lib::serial::write_string("\n");

    if let Some(loc) = location {
        std_lib::serial::write_string("Location: ");
        std_lib::serial::write_string(loc.file());
        std_lib::serial::write_string(":");
        write_hex(loc.line() as u64);
        std_lib::serial::write_string(":");
        write_hex(loc.column() as u64);
        std_lib::serial::write_string("\n");
    } else {
        std_lib::serial::write_string("Location: unknown\n");
    }

    std_lib::serial::write_string(PANIC_SEPARATOR);
    std_lib::serial::write_string("\n\n");
}

fn print_vga_header(info: &PanicInfo) {
    use crate::std_lib::vga::{println_with_color, ColorCodeVga};

    println_with_color(
        VGA_PANIC_SEPARATOR,
        ColorCodeVga::LightRed,
        ColorCodeVga::Black,
    );
    println_with_color(
        "           KERNEL PANIC                ",
        ColorCodeVga::LightRed,
        ColorCodeVga::Black,
    );
    println_with_color(
        VGA_PANIC_SEPARATOR,
        ColorCodeVga::LightRed,
        ColorCodeVga::Black,
    );

    if let Some(loc) = info.location() {
        let msg = format!("{}:{}:{}", loc.file(), loc.line(), loc.column());
        println_with_color(&msg, ColorCodeVga::Yellow, ColorCodeVga::Black);
    }
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    print_vga_header(info);
    print_panic_header(info);

    std_lib::serial::write_string("\n");
    std_lib::serial::write_string(MSG_SYSTEM_HALTED);
    std_lib::serial::write_string("\n");

    use crate::std_lib::vga::println_with_color;
    use crate::std_lib::vga::ColorCodeVga;
    println_with_color(
        MSG_SYSTEM_HALTED,
        ColorCodeVga::LightRed,
        ColorCodeVga::Black,
    );
    println_with_color(MSG_QEMU_EXIT, ColorCodeVga::LightRed, ColorCodeVga::Black);

    loop {
        x86_64::instructions::hlt();
    }
}
