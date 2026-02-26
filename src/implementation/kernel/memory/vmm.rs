use crate::drivers::{serial, vga};
use x86_64::structures::idt::InterruptStackFrame;

pub fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) {
    use x86_64::registers::control::Cr2;

    let fault_addr = match Cr2::read() {
        Ok(addr) => addr.as_u64(),
        Err(_) => 0,
    };

    vga::println("PAGE FAULT!");
    serial::write_string("PAGE FAULT!\n");

    vga::print("Address: 0x");
    serial::write_string("Address: 0x");
    vga::println_hex(fault_addr);
    serial::write_hex(fault_addr);
    serial::write_string("\n");

    vga::print("Error code: 0x");
    serial::write_string("Error code: 0x");
    vga::println_hex(error_code);
    serial::write_hex(error_code);
    serial::write_string("\n");

    vga::print("IP: 0x");
    serial::write_string("IP: 0x");
    vga::println_hex(stack_frame.instruction_pointer.as_u64());
    serial::write_hex(stack_frame.instruction_pointer.as_u64());
    serial::write_string("\n");

    if error_code & 1 != 0 {
        vga::println("Cause: Protection violation");
        serial::write_string("Cause: Protection violation\n");
    } else {
        vga::println("Cause: Non-present page");
        serial::write_string("Cause: Non-present page\n");
    }

    if error_code & 2 != 0 {
        vga::println("Write access");
        serial::write_string("Write access\n");
    } else {
        vga::println("Read access");
        serial::write_string("Read access\n");
    }

    if error_code & 4 != 0 {
        vga::println("User mode");
        serial::write_string("User mode\n");
    } else {
        vga::println("Kernel mode");
        serial::write_string("Kernel mode\n");
    }

    loop {
        x86_64::instructions::hlt();
    }
}
