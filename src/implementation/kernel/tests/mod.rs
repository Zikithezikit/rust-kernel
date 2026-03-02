pub mod test_allocator;
pub mod test_panic;
pub mod test_pmm;
pub mod test_syscall;
pub mod test_task;
pub mod test_tss;
pub mod test_vfs;
pub mod test_vmm;

use crate::{drivers::serial, task::scheduler::SCHEDULER};

/// Runs all kernel tests.
///
/// NOTE: This function is currently not called from main() due to a kernel
/// reset issue that occurs after test completion. This needs to be investigated.
pub fn run_tests() {
    serial::write_string("Running tests...\n");
    serial::write_string("=== Running kernel tests ===\n");

    // =========================
    // Allocator Tests
    // =========================
    serial::write_string("\n=== Allocator Tests ===\n");
    test_allocator::test_vec_allocation();
    test_allocator::test_box_allocation();
    test_allocator::test_string_allocation();
    test_allocator::test_multiple_allocations();
    serial::write_string("=== Allocator: OK ===\n");

    // =========================
    // Panic Tests
    // =========================
    serial::write_string("\n=== Panic Tests ===\n");
    test_panic::test_panic_constants();
    test_panic::test_serial_hex_output();
    serial::write_string("=== Panic: OK ===\n");

    // =========================
    // Physical Memory Manager (PMM) Tests
    // =========================
    serial::write_string("\n=== PMM Tests ===\n");
    test_pmm::test_pmm_initialized();
    test_pmm::test_pmm_allocate_single_page();
    test_pmm::test_pmm_allocate_multiple_pages();
    test_pmm::test_pmm_stress();
    serial::write_string("=== PMM: OK ===\n");

    // =========================
    // Virtual Memory Manager (VMM) Tests
    // =========================
    serial::write_string("\n=== VMM Tests ===\n");
    test_vmm::test_page_fault_handler();
    serial::write_string("=== VMM: OK ===\n");

    // =========================
    // Syscall Tests
    // =========================
    serial::write_string("\n=== Syscall Tests ===\n");
    test_syscall::test_syscall_table_initialized();
    test_syscall::test_syscall_numbers();
    test_syscall::test_system_ticks();
    test_syscall::test_syscall_handler_exists();
    test_syscall::test_file_descriptor_enum();
    test_syscall::test_errno_enum();
    test_syscall::test_syscall_number_from_u32();
    serial::write_string("=== Syscall: OK ===\n");

    // =========================
    // Task & Scheduler Tests  = This is currently commented out because we are still in a single thread kernel. when we try to context switch we corupt in the assembly code the stack pointer and so we crash the kernel. 
    // =========================
    // serial::write_string("\n=== Task & Scheduler Tests ===\n")

    // // Task creation & ID allocator tests (safe even without reset)
    // test_task::test_task_creation();
    // test_task::test_id_allocator_basic();
    // test_task::test_id_allocator_reuse();
    // test_task::test_id_allocator_max();

    // // Scheduler initialization & tick tests
    // test_task::test_scheduler_init();
    // test_task::test_task_spawn();
    // test_task::test_task_state_transitions();
    // test_task::test_task_is_runnable();

    // // Scheduler tick & preemptive tests
    // test_task::test_scheduler_tick();
    // test_task::test_scheduler_tick_time_slice();
    // test_task::test_scheduler_preemptive_tick();
    // serial::write_string("=== Task & Scheduler: OK ===\n");

    // =========================
    // TSS & GDT Tests
    // =========================
    serial::write_string("\n=== TSS & GDT Tests ===\n");
    test_tss::test_tss_initialized();
    test_tss::test_tss_double_fault_stack();
    test_tss::test_gdt_has_code_segment();
    test_tss::test_tss_ist_index();
    serial::write_string("=== TSS & GDT: OK ===\n");

    // =========================
    // VFS Core Tests
    // =========================
    serial::write_string("\n=== VFS Core Tests ===\n");
    test_vfs::test_vfs_inode_creation();
    test_vfs::test_vfs_directory_inode();
    test_vfs::test_vfs_inode_mkdir();
    test_vfs::test_vfs_inode_create_file();
    test_vfs::test_vfs_inode_read_write();
    test_vfs::test_vfs_inode_unlink();
    test_vfs::test_vfs_inode_rmdir();
    test_vfs::test_vfs_file_table();
    test_vfs::test_vfs_file_read_write();
    test_vfs::test_vfs_mount_namespace();
    test_vfs::test_vfs_dentry();
    test_vfs::test_vfs_dentry_lookup();
    test_vfs::test_vfs_super_block();
    test_vfs::test_vfs_symlink();
    test_vfs::test_vfs_readlink();
    test_vfs::test_vfs_file_seek();
    test_vfs::test_vfs_file_truncate();
    test_vfs::test_vfs_file_dup();
    test_vfs::test_vfs_readdir();
    test_vfs::test_vfs_mount_unmount();
    test_vfs::test_vfs_walk_path();
    serial::write_string("=== VFS Core: OK ===\n");

    // =========================
    // Initramfs Tests
    // =========================
    serial::write_string("\n=== Initramfs Tests ===\n");
    test_vfs::test_initramfs_root_creation();
    test_vfs::test_initramfs_children();
    test_vfs::test_initramfs_file_read();
    test_vfs::test_initramfs_readdir();
    serial::write_string("=== Initramfs: OK ===\n");

    // =========================
    // Tmpfs Tests
    // =========================
    serial::write_string("\n=== Tmpfs Tests ===\n");
    test_vfs::test_tmpfs_root_creation();
    test_vfs::test_tmpfs_children();
    test_vfs::test_tmpfs_mkdir();
    test_vfs::test_tmpfs_create_file();
    test_vfs::test_tmpfs_read_write();
    test_vfs::test_tmpfs_unlink();
    test_vfs::test_tmpfs_rmdir();
    test_vfs::test_tmpfs_readdir();
    test_vfs::test_tmpfs_truncate();
    test_vfs::test_tmpfs_symlink();
    test_vfs::test_tmpfs_readlink();
    serial::write_string("=== Tmpfs: OK ===\n");

    serial::write_string("All tests completed!\n");
    serial::write_string("=== Tests completed ===\n");

}

