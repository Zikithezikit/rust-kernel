/// Tests that the allocator works correctly with Vec.
pub fn test_vec_allocation() {
    use crate::alloc::vec::Vec;
    use crate::std_lib::serial;

    serial::write_string("Vec allocation test: ");

    let mut numbers: Vec<u32> = Vec::new();
    numbers.push(42);
    numbers.push(123);
    numbers.push(999);

    for n in &numbers {
        let s = crate::alloc::format!("{} ", n);
        serial::write_string(&s);
    }
    serial::write_string("\n");
}

/// Tests that the allocator works correctly with Box.
pub fn test_box_allocation() {
    use crate::alloc::boxed::Box;
    use crate::std_lib::serial;

    serial::write_string("Box allocation test: ");

    let boxed = Box::new(42);
    let s = crate::alloc::format!("{} ", *boxed);
    serial::write_string(&s);

    serial::write_string("\n");
}

/// Tests that the allocator works correctly with String.
pub fn test_string_allocation() {
    use crate::alloc::string::String;
    use crate::std_lib::serial;

    serial::write_string("String allocation test: ");

    let mut s = String::new();
    s.push_str("Hello, ");
    s.push_str("Kernel!");

    serial::write_string(&s);
    serial::write_string("\n");
}

/// Tests multiple allocations in sequence.
pub fn test_multiple_allocations() {
    use crate::alloc::boxed::Box;
    use crate::alloc::string::String;
    use crate::alloc::vec::Vec;
    use crate::std_lib::serial;

    serial::write_string("Multiple allocations test: ");

    let _vec: Vec<u8> = Vec::with_capacity(10);
    let _string = String::from("test");
    let _box = Box::new(123);

    serial::write_string("OK");
    serial::write_string("\n");
}
