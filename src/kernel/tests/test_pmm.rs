use crate::drivers::serial;
use crate::mm::pmm::PMM;

pub fn test_pmm_initialized() {
    let total = PMM.get_total_pages();
    let free = PMM.get_free_pages();

    serial::write_string("PMM Test: Total pages = ");
    serial::write_hex(total as u64);
    serial::write_string(", Free pages = ");
    serial::write_hex(free as u64);
    serial::write_string("\n");

    let ok = total > 0 && free > 0 && free <= total;
    serial::write_string(if ok {
        "pmm_init: OK\n"
    } else {
        "pmm_init: FAIL\n"
    });
}

pub fn test_pmm_allocate_single_page() {
    let addr = PMM.allocate_page();
    match addr {
        Some(page_addr) => {
            serial::write_string("Allocated page at: ");
            serial::write_hex(page_addr as u64);
            serial::write_string("\n");

            let free_before = PMM.get_free_pages();
            PMM.deallocate_page(page_addr);
            let free_after = PMM.get_free_pages();

            let ok = free_after == free_before + 1;
            serial::write_string(if ok {
                "pmm_alloc_single: OK\n"
            } else {
                "pmm_alloc_single: FAIL\n"
            });
        }
        None => {
            serial::write_string("pmm_alloc_single: FAIL (no pages available)\n");
        }
    }
}

pub fn test_pmm_allocate_multiple_pages() {
    let num_pages = 4;
    let addr = PMM.allocate_pages(num_pages);
    match addr {
        Some(page_addr) => {
            serial::write_string("Allocated ");
            serial::write_hex(num_pages as u64);
            serial::write_string(" pages at: ");
            serial::write_hex(page_addr as u64);
            serial::write_string("\n");

            let free_before = PMM.get_free_pages();
            PMM.deallocate_pages(page_addr, num_pages);
            let free_after = PMM.get_free_pages();

            let ok = free_after == free_before + num_pages;
            serial::write_string(if ok {
                "pmm_alloc_multi: OK\n"
            } else {
                "pmm_alloc_multi: FAIL\n"
            });
        }
        None => {
            serial::write_string("pmm_alloc_multi: FAIL (no pages available)\n");
        }
    }
}

pub fn test_pmm_stress() {
    let mut pages: [Option<usize>; 16] = [None; 16];

    for i in 0..16 {
        pages[i] = PMM.allocate_page();
        if pages[i].is_none() {
            serial::write_string("pmm_stress: FAIL (allocation failed at page ");
            serial::write_hex(i as u64);
            serial::write_string(")\n");
            return;
        }
    }

    for i in 0..16 {
        if let Some(addr) = pages[i] {
            PMM.deallocate_page(addr);
        }
    }

    serial::write_string("pmm_stress: OK\n");
}
