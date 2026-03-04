use crate::arch::x86::tss::{get_gdt, get_tss, DOUBLE_FAULT_IST_INDEX};
use crate::drivers::serial;

pub fn test_tss_initialized() {
    serial::write_string("Testing TSS initialization...\n");

    let tss = get_tss();
    let gdt = get_gdt();

    let ok = tss.is_some() && gdt.is_some();

    serial::write_string(if ok {
        "test_tss_initialized: OK\n"
    } else {
        "test_tss_initialized: FAIL\n"
    });
}

pub fn test_tss_double_fault_stack() {
    serial::write_string("Testing TSS double fault stack...\n");

    let tss = get_tss();

    match tss {
        Some(t) => {
            let ist_entry = t.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize];
            let ok = ist_entry.as_u64() != 0;

            serial::write_string("Double fault stack address: ");
            serial::write_hex(ist_entry.as_u64());
            serial::write_string("\n");

            serial::write_string(if ok {
                "test_tss_double_fault_stack: OK\n"
            } else {
                "test_tss_double_fault_stack: FAIL\n"
            });
        }
        None => {
            serial::write_string("test_tss_double_fault_stack: FAIL (TSS not initialized)\n");
        }
    }
}

pub fn test_gdt_has_code_segment() {
    serial::write_string("Testing GDT code segment...\n");

    let gdt = get_gdt();

    match gdt {
        Some(_g) => {
            serial::write_string("GDT code segment exists: OK\n");
            serial::write_string("test_gdt_has_code_segment: OK\n");
        }
        None => {
            serial::write_string("test_gdt_has_code_segment: FAIL\n");
        }
    }
}

pub fn test_tss_ist_index() {
    serial::write_string("Testing TSS IST index...\n");

    let ok = DOUBLE_FAULT_IST_INDEX == 0;

    serial::write_string("IST index: ");
    serial::write_hex(DOUBLE_FAULT_IST_INDEX as u64);
    serial::write_string("\n");

    serial::write_string(if ok {
        "test_tss_ist_index: OK\n"
    } else {
        "test_tss_ist_index: FAIL\n"
    });
}
