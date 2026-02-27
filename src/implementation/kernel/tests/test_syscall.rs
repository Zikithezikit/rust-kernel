use crate::drivers::serial;
use crate::syscall::handler::{get_ticks, increment_ticks, SYSCALL_TABLE};
use crate::syscall::numbers::{NR_syscalls, SyscallNumber};

pub fn test_syscall_table_initialized() {
    serial::write_string("Testing syscall table initialized...\n");

    let mut count = 0;
    for i in 0..NR_syscalls {
        if SYSCALL_TABLE[i].is_some() {
            count += 1;
        }
    }

    serial::write_string("Registered syscalls: ");
    serial::write_hex(count as u64);
    serial::write_string("\n");

    let ok = count > 0;
    serial::write_string(if ok {
        "syscall_table_initialized: OK\n"
    } else {
        "syscall_table_initialized: FAIL\n"
    });
}

pub fn test_syscall_numbers() {
    serial::write_string("Testing syscall numbers...\n");

    let exit_num = SyscallNumber::SysExit as usize;
    let write_num = SyscallNumber::SysWrite as usize;
    let read_num = SyscallNumber::SysRead as usize;
    let getpid_num = SyscallNumber::SysGetPid as usize;
    let fork_num = SyscallNumber::SysFork as usize;
    let exec_num = SyscallNumber::SysExecve as usize;

    serial::write_string("SysExit: ");
    serial::write_hex(exit_num as u64);
    serial::write_string("\n");

    serial::write_string("SysWrite: ");
    serial::write_hex(write_num as u64);
    serial::write_string("\n");

    serial::write_string("SysRead: ");
    serial::write_hex(read_num as u64);
    serial::write_string("\n");

    serial::write_string("SysGetPid: ");
    serial::write_hex(getpid_num as u64);
    serial::write_string("\n");

    serial::write_string("SysFork: ");
    serial::write_hex(fork_num as u64);
    serial::write_string("\n");

    serial::write_string("SysExecve: ");
    serial::write_hex(exec_num as u64);
    serial::write_string("\n");

    let ok = exit_num == 0 && write_num == 1 && read_num == 2 && getpid_num == 5;
    serial::write_string(if ok {
        "syscall_numbers: OK\n"
    } else {
        "syscall_numbers: FAIL\n"
    });
}

pub fn test_system_ticks() {
    serial::write_string("Testing system ticks...\n");

    let initial_ticks = get_ticks();
    serial::write_string("Initial ticks: ");
    serial::write_hex(initial_ticks);
    serial::write_string("\n");

    increment_ticks();

    let after_ticks = get_ticks();
    serial::write_string("After increment: ");
    serial::write_hex(after_ticks);
    serial::write_string("\n");

    increment_ticks();
    increment_ticks();
    increment_ticks();

    let final_ticks = get_ticks();
    serial::write_string("Final ticks: ");
    serial::write_hex(final_ticks);
    serial::write_string("\n");

    let ok = initial_ticks == 0 && after_ticks == 1 && final_ticks == 4;
    serial::write_string(if ok {
        "system_ticks: OK\n"
    } else {
        "system_ticks: FAIL\n"
    });
}

pub fn test_syscall_handler_exists() {
    serial::write_string("Testing syscall handler exists...\n");

    let exit_fn = SYSCALL_TABLE[0];
    let write_fn = SYSCALL_TABLE[1];
    let read_fn = SYSCALL_TABLE[2];
    let getpid_fn = SYSCALL_TABLE[5];

    let ok = exit_fn.is_some() && write_fn.is_some() && read_fn.is_some() && getpid_fn.is_some();

    serial::write_string(if ok {
        "syscall_handler_exists: OK\n"
    } else {
        "syscall_handler_exists: FAIL\n"
    });
}
