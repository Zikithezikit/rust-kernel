use crate::std_lib::serial;

const TEST_ADDRESS: usize = 0x100000;
const TEST_VAL: u8 = 0x42;

pub fn test_page_fault_handler() {
    serial::write_string("\n=== VMM Tests ===\n");
    test_hex();
    test_paging();
    serial::write_string("=== VMM: OK ===\n");
}

fn test_hex() {
    let ptr = TEST_ADDRESS as *mut u8;
    unsafe {
        ptr.write_volatile(TEST_VAL);
        let val = ptr.read_volatile();
        serial::write_string(if val == TEST_VAL {
            "hex: OK\n"
        } else {
            "hex: FAIL\n"
        });
    }
}

fn test_paging() {
    let ptr = TEST_ADDRESS as *mut u8;
    unsafe {
        ptr.write_volatile(0xAB);
        let val = ptr.read_volatile();
        serial::write_string(if val == 0xAB {
            "paging: OK\n"
        } else {
            "paging: FAIL\n"
        });
    }
}
