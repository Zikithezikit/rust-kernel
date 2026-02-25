mod test_allocator;

/// Runs all kernel tests.
pub fn run_tests() {
    test_allocator::test_vec_allocation();
    test_allocator::test_box_allocation();
    test_allocator::test_string_allocation();
    test_allocator::test_multiple_allocations();
}
