mod test_allocator;
mod test_panic;
mod test_pmm;
mod test_vmm;

pub fn run_tests() {
    test_allocator::test_vec_allocation();
    test_allocator::test_box_allocation();
    test_allocator::test_string_allocation();
    test_allocator::test_multiple_allocations();
    test_panic::test_panic_constants();
    test_panic::test_serial_hex_output();
    test_pmm::test_pmm_initialized();
    test_pmm::test_pmm_allocate_single_page();
    test_pmm::test_pmm_allocate_multiple_pages();
    test_pmm::test_pmm_stress();
    test_vmm::test_page_fault_handler();
}
