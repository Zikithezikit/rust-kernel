use crate::std_lib::serial;

pub fn test_vec_allocation() {
    use crate::alloc::vec::Vec;
    let mut v: Vec<u32> = Vec::new();
    v.push(42);
    v.push(123);
    v.push(999);
    let ok = v.len() == 3 && v[0] == 42 && v[1] == 123 && v[2] == 999;
    serial::write_string(if ok { "vec: OK\n" } else { "vec: FAIL\n" });
}

pub fn test_box_allocation() {
    use crate::alloc::boxed::Box;
    let b = Box::new(42);
    serial::write_string(if *b == 42 { "box: OK\n" } else { "box: FAIL\n" });
}

pub fn test_string_allocation() {
    use crate::alloc::string::String;
    let mut s = String::new();
    s.push_str("Hello, Kernel!");
    serial::write_string(if s == "Hello, Kernel!" {
        "string: OK\n"
    } else {
        "string: FAIL\n"
    });
}

pub fn test_multiple_allocations() {
    use crate::alloc::boxed::Box;
    use crate::alloc::string::String;
    use crate::alloc::vec::Vec;

    let _v: Vec<u8> = Vec::with_capacity(10);
    let _s = String::from("test");
    let _b = Box::new(123);
    serial::write_string("mixed: OK\n");
}
