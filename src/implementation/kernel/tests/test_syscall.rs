use crate::drivers::serial;
use crate::syscall::handler::{get_ticks, increment_ticks, SYSCALL_TABLE};
use crate::syscall::numbers::{Errno, FileDescriptor, NR_syscalls, SyscallNumber, SyscallResult};

pub fn test_syscall_table_initialized() {
    serial::write_string("Testing syscall table initialized...\n");

    let mut count: usize = 0;
    for i in 0..NR_syscalls {
        if SYSCALL_TABLE[i].is_some() {
            count += 1;
        }
    }

    serial::write_string("Registered syscalls: ");
    serial::write_hex(count as u64);
    serial::write_string("\n");

    let ok: bool = count > 0;
    serial::write_string(if ok {
        "syscall_table_initialized: OK\n"
    } else {
        "syscall_table_initialized: FAIL\n"
    });
}

pub fn test_syscall_numbers() {
    serial::write_string("Testing syscall numbers...\n");

    let exit_num: usize = SyscallNumber::SysExit as usize;
    let write_num: usize = SyscallNumber::SysWrite as usize;
    let read_num: usize = SyscallNumber::SysRead as usize;
    let getpid_num: usize = SyscallNumber::SysGetPid as usize;
    let fork_num: usize = SyscallNumber::SysFork as usize;
    let exec_num: usize = SyscallNumber::SysExecve as usize;

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

    let ok: bool = exit_num == 0 && write_num == 1 && read_num == 2 && getpid_num == 5;
    serial::write_string(if ok {
        "syscall_numbers: OK\n"
    } else {
        "syscall_numbers: FAIL\n"
    });
}

pub fn test_system_ticks() {
    serial::write_string("Testing system ticks...\n");

    let initial_ticks: u64 = get_ticks();
    serial::write_string("Initial ticks: ");
    serial::write_hex(initial_ticks);
    serial::write_string("\n");

    increment_ticks();

    let after_ticks: u64 = get_ticks();
    serial::write_string("After increment: ");
    serial::write_hex(after_ticks);
    serial::write_string("\n");

    increment_ticks();
    increment_ticks();
    increment_ticks();

    let final_ticks: u64 = get_ticks();
    serial::write_string("Final ticks: ");
    serial::write_hex(final_ticks);
    serial::write_string("\n");

    let ok: bool = initial_ticks == 0 && after_ticks == 1 && final_ticks == 4;
    serial::write_string(if ok {
        "system_ticks: OK\n"
    } else {
        "system_ticks: FAIL\n"
    });
}

pub fn test_syscall_handler_exists() {
    serial::write_string("Testing syscall handler exists...\n");

    let exit_fn = SYSCALL_TABLE[SyscallNumber::SysExit as usize];
    let write_fn = SYSCALL_TABLE[SyscallNumber::SysWrite as usize];
    let read_fn = SYSCALL_TABLE[SyscallNumber::SysRead as usize];
    let getpid_fn = SYSCALL_TABLE[SyscallNumber::SysGetPid as usize];

    let ok: bool =
        exit_fn.is_some() && write_fn.is_some() && read_fn.is_some() && getpid_fn.is_some();

    serial::write_string(if ok {
        "syscall_handler_exists: OK\n"
    } else {
        "syscall_handler_exists: FAIL\n"
    });
}

pub fn test_file_descriptor_enum() {
    serial::write_string("Testing FileDescriptor enum...\n");

    let stdin_val: usize = FileDescriptor::Stdin as usize;
    let stdout_val: usize = FileDescriptor::Stdout as usize;
    let stderr_val: usize = FileDescriptor::Stderr as usize;

    serial::write_string("Stdin: ");
    serial::write_hex(stdin_val as u64);
    serial::write_string("\n");

    serial::write_string("Stdout: ");
    serial::write_hex(stdout_val as u64);
    serial::write_string("\n");

    serial::write_string("Stderr: ");
    serial::write_hex(stderr_val as u64);
    serial::write_string("\n");

    let from_stdin: Option<FileDescriptor> = FileDescriptor::from_usize(0);
    let from_stdout: Option<FileDescriptor> = FileDescriptor::from_usize(1);
    let from_stderr: Option<FileDescriptor> = FileDescriptor::from_usize(2);
    let from_invalid: Option<FileDescriptor> = FileDescriptor::from_usize(99);

    let ok: bool = from_stdin == Some(FileDescriptor::Stdin)
        && from_stdout == Some(FileDescriptor::Stdout)
        && from_stderr == Some(FileDescriptor::Stderr)
        && from_invalid == None;

    serial::write_string(if ok {
        "file_descriptor_enum: OK\n"
    } else {
        "file_descriptor_enum: FAIL\n"
    });
}

pub fn test_errno_enum() {
    serial::write_string("Testing Errno enum...\n");

    let einval_val: SyscallResult = Errno::EINVAL.as_isize();
    let ebadf_val: SyscallResult = Errno::EBADF.as_isize();
    let eagain_val: SyscallResult = Errno::EAGAIN.as_isize();
    let enomem_val: SyscallResult = Errno::ENOMEM.as_isize();
    let enosys_val: SyscallResult = Errno::ENOSYS.as_isize();
    let eok_val: SyscallResult = Errno::Ok.as_isize();

    serial::write_string("EINVAL: ");
    serial::write_hex(einval_val as u64);
    serial::write_string("\n");

    serial::write_string("EBADF: ");
    serial::write_hex(ebadf_val as u64);
    serial::write_string("\n");

    serial::write_string("EAGAIN: ");
    serial::write_hex(eagain_val as u64);
    serial::write_string("\n");

    serial::write_string("ENOMEM: ");
    serial::write_hex(enomem_val as u64);
    serial::write_string("\n");

    serial::write_string("ENOSYS: ");
    serial::write_hex(enosys_val as u64);
    serial::write_string("\n");

    serial::write_string("Ok: ");
    serial::write_hex(eok_val as u64);
    serial::write_string("\n");

    let ok: bool = einval_val == -22
        && ebadf_val == -9
        && eagain_val == -11
        && enomem_val == -12
        && enosys_val == -38
        && eok_val == 0;

    serial::write_string(if ok {
        "errno_enum: OK\n"
    } else {
        "errno_enum: FAIL\n"
    });
}

pub fn test_syscall_number_from_u32() {
    serial::write_string("Testing SyscallNumber::from_u32...\n");

    let valid_0: Option<SyscallNumber> = SyscallNumber::from_u32(0);
    let valid_1: Option<SyscallNumber> = SyscallNumber::from_u32(1);
    let valid_5: Option<SyscallNumber> = SyscallNumber::from_u32(5);
    let valid_23: Option<SyscallNumber> = SyscallNumber::from_u32(23);
    let invalid_24: Option<SyscallNumber> = SyscallNumber::from_u32(24);
    let invalid_100: Option<SyscallNumber> = SyscallNumber::from_u32(100);

    serial::write_string("from_u32(0): ");
    serial::write_hex(valid_0.map(|v| v as u32).unwrap_or(0) as u64);
    serial::write_string("\n");

    serial::write_string("from_u32(24): ");
    serial::write_hex(invalid_24.map(|v| v as u32).unwrap_or(0) as u64);
    serial::write_string("\n");

    let ok: bool = valid_0 == Some(SyscallNumber::SysExit)
        && valid_1 == Some(SyscallNumber::SysWrite)
        && valid_5 == Some(SyscallNumber::SysGetPid)
        && valid_23 == Some(SyscallNumber::SysSetgid)
        && invalid_24 == None
        && invalid_100 == None;

    serial::write_string(if ok {
        "syscall_number_from_u32: OK\n"
    } else {
        "syscall_number_from_u32: FAIL\n"
    });
}
