mod test_allocator;
mod test_vmm;

pub fn run_tests() {
    test_allocator::test_vec_allocation();
    test_allocator::test_box_allocation();
    test_allocator::test_string_allocation();
    test_allocator::test_multiple_allocations();
    test_vmm::test_page_fault_handler();
}
