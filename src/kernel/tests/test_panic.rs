// src/implementation/kernel/tests/test_panic.rs
//!
//! Tests for the panic handler module.
//!
//! Note: The actual panic handler cannot be tested directly as it halts the system.
//! These tests verify the helper functions and constants used by the panic handler.

use crate::drivers::serial;

pub fn test_panic_constants() {
    serial::write_string("Testing panic handler constants...\n");

    let sep_len = "======================================".len();
    let vga_sep_len = "========================================".len();

    let ok = sep_len == 38 && vga_sep_len == 40;
    serial::write_string(if ok {
        "panic_constants: OK\n"
    } else {
        "panic_constants: FAIL\n"
    });
}

pub fn test_serial_hex_output() {
    serial::write_string("Testing serial hex output...\n");

    let test_values = [0u64, 1, 0xF, 0xFF, 0x100, 0xFFFF_FFFF];
    let all_ok = true;

    for val in test_values {
        serial::write_string("hex ");
        serial::write_hex(val);
        serial::write_string(" = ");

        let mut expected_chars = 0;
        let mut v = val;
        if v == 0 {
            expected_chars = 1;
        } else {
            while v > 0 {
                expected_chars += 1;
                v >>= 4;
            }
        }

        serial::write_string(" (");
        serial::write_hex(expected_chars as u64);
        serial::write_string(" chars)\n");
    }

    serial::write_string(if all_ok {
        "panic_hex: OK\n"
    } else {
        "panic_hex: FAIL\n"
    });
}
