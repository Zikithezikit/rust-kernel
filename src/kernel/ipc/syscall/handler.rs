//! System call handler
//!
//! Handles system call interrupts (int 0x80).
//! Uses Linux-style syscall table and calling convention:
//! - syscall number in eax/rax
//! - arguments in ebx/rdi, ecx/rsi, edx/rdx, esi/r10, edi/r8, ebp/r9
//! - return value in eax/rax

use crate::drivers::serial;
use crate::drivers::vga;
use crate::kernel::id_allocator::TASK_ID_ALLOCATOR;
use crate::kernel::scheduler::SCHEDULER;
use crate::kernel::task::Task;
use crate::kernel::task::{TaskState, KERNEL_STACK_SIZE};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

pub use crate::ipc::syscall::numbers::{
    Errno, FileDescriptor, SyscallFn, SyscallNumber, SyscallResult, NR_SYSCALLS,
};

/// Default time slice for forked tasks (in ticks)
const FORK_TIME_SLICE: usize = 10;

/// Syscall table - maps syscall numbers to functions
/// Similar to Linux's sys_call_table
pub static SYSCALL_TABLE: [Option<SyscallFn>; NR_SYSCALLS] = {
    let mut table: [Option<SyscallFn>; NR_SYSCALLS] = [None; NR_SYSCALLS];
    table[SyscallNumber::SysExit as usize] = Some(sys_exit);
    table[SyscallNumber::SysWrite as usize] = Some(sys_write);
    table[SyscallNumber::SysRead as usize] = Some(sys_read);
    table[SyscallNumber::SysOpen as usize] = Some(sys_open);
    table[SyscallNumber::SysClose as usize] = Some(sys_close);
    table[SyscallNumber::SysGetPid as usize] = Some(sys_getpid);
    table[SyscallNumber::SysGetPpid as usize] = Some(sys_getppid);
    table[SyscallNumber::SysFork as usize] = Some(sys_fork);
    table[SyscallNumber::SysExecve as usize] = Some(sys_execve);
    table[SyscallNumber::SysGetuid as usize] = Some(sys_getuid);
    table[SyscallNumber::SysGetgid as usize] = Some(sys_getgid);
    table[SyscallNumber::SysSetuid as usize] = Some(sys_setuid);
    table[SyscallNumber::SysSetgid as usize] = Some(sys_setgid);
    table
};

/// Global system ticks since boot
static mut SYSTEM_TICKS: u64 = 0;

/// Increment system ticks (called from timer interrupt)
pub fn increment_ticks() {
    unsafe {
        SYSTEM_TICKS += 1;
    }
}

/// Get current system ticks
pub fn get_ticks() -> u64 {
    unsafe { SYSTEM_TICKS }
}

/// Read syscall arguments from registers using inline asm
macro_rules! get_syscall_args {
    ($num:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident) => {
        core::arch::asm!(
            "mov {0}, rax",
            "mov {1}, rdi",
            "mov {2}, rsi",
            "mov {3}, rdx",
            "mov {4}, r10",
            "mov {5}, r8",
            "mov {6}, r9",
            out(reg) $num,
            out(reg) $a1,
            out(reg) $a2,
            out(reg) $a3,
            out(reg) $a4,
            out(reg) $a5,
            out(reg) $a6,
        );
    }
}

/// System call handler
///
/// Called via int 0x80. Uses Linux x86-64 calling convention.
pub unsafe fn handle_syscall(stack_frame: &InterruptStackFrame) -> SyscallResult {
    let syscall_num: usize;
    let arg1: usize;
    let arg2: usize;
    let arg3: usize;
    let arg4: usize;
    let arg5: usize;
    let arg6: usize;

    get_syscall_args!(syscall_num, arg1, arg2, arg3, arg4, arg5, arg6);

    // Check if coming from user mode
    let cs = stack_frame.code_segment;
    if (cs.0 & 3) == 3 {
        // Limited logging to avoid flooding
        if syscall_num != SyscallNumber::SysWrite as usize || get_ticks() % 100 == 0 {
            serial::write_string(&format!("USER syscall: {}\n", syscall_num));
        }
    }

    if syscall_num >= NR_SYSCALLS {
        serial::write_string(&format!("syscall: {} out of range\n", syscall_num));
        return -(SyscallResult::MAX as SyscallResult);
    }

    let syscall_fn: Option<SyscallFn> = SYSCALL_TABLE[syscall_num];

    match syscall_fn {
        Some(fn_ptr) => {
            let result: SyscallResult = fn_ptr(arg1, arg2, arg3, arg4, arg5, arg6);
            result
        }
        None => {
            serial::write_string(&format!("syscall: {} not implemented\n", syscall_num));
            -(SyscallResult::MAX as SyscallResult)
        }
    }
}

// ============================================================================
// Syscall implementations
// ============================================================================

/// sys_exit - terminate current process
fn sys_exit(
    arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    let exit_code = arg1 as i32;

    serial::write_string(&format!("sys_exit: code={}\n", exit_code));

    if let Some(current) = SCHEDULER.current_task() {
        let task_id = current.lock().id;
        current.lock().state = TaskState::Zombie;
        current.lock().free_kernel_stack();
        SCHEDULER.remove_task(task_id);
    }

    loop {
        x86_64::instructions::hlt();
    }
}

/// sys_write - write to file descriptor
fn sys_write(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    let fd: usize = arg1;
    let buf: *const u8 = arg2 as *const u8;
    let count: usize = arg3;

    if buf.is_null() {
        return Errno::EINVAL.as_isize();
    }

    let slice: &[u8] = unsafe { core::slice::from_raw_parts(buf, count) };

    for &byte in slice {
        if byte == b'\n' {
            vga::println("");
        } else if byte != b'\r' {
            vga::print_char(byte);
        }
    }

    serial::write_string(&String::from_utf8_lossy(slice));

    if let Some(fd_enum) = FileDescriptor::from_usize(fd) {
        match fd_enum {
            FileDescriptor::Stdout | FileDescriptor::Stderr => count as SyscallResult,
            _ => Errno::EBADF.as_isize(),
        }
    } else {
        Errno::EBADF.as_isize()
    }
}

/// sys_read - read from file descriptor
fn sys_read(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}

/// sys_getpid - get current process ID
fn sys_getpid(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    if let Some(current) = SCHEDULER.current_task() {
        current.lock().id as SyscallResult
    } else {
        Errno::Ok.as_isize()
    }
}

/// sys_getuid - get current user ID
fn sys_getuid(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}

/// sys_getgid - get current group ID
fn sys_getgid(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}

/// sys_fork - create a child process
fn sys_fork(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    if let Some(current) = SCHEDULER.current_task() {
        let parent_id: usize = current.lock().id;
        let parent_ip: usize = current.lock().instruction_pointer;

        let new_id: usize = match TASK_ID_ALLOCATOR.alloc() {
            Some(id) => id,
            None => return Errno::EAGAIN.as_isize(),
        };

        let stack: usize = {
            use crate::mm::pmm::PMM;
            let pages_needed: usize = KERNEL_STACK_SIZE / crate::mm::pmm::PAGE_SIZE;
            match PMM.allocate_pages(pages_needed) {
                Some(addr) => addr + KERNEL_STACK_SIZE,
                None => {
                    unsafe {
                        TASK_ID_ALLOCATOR.free(new_id);
                    }
                    return Errno::EAGAIN.as_isize();
                }
            }
        };

        let child_cr3 = current.lock().cr3;

        let child = Arc::new(Mutex::new(Task {
            id: new_id,
            parent_id,
            state: TaskState::Ready,
            kernel_stack: stack,
            kernel_stack_top: stack,
            stack_size: KERNEL_STACK_SIZE,
            cr3: child_cr3,
            instruction_pointer: parent_ip,
            time_slice: FORK_TIME_SLICE,
            time_slice_max: FORK_TIME_SLICE,
            ticks_run: 0,
            name: "forked",
        }));

        SCHEDULER.add_task(child);

        serial::write_string(&format!(
            "sys_fork: parent={}, child={}\n",
            parent_id, new_id
        ));

        new_id as SyscallResult
    } else {
        Errno::ENOMEM.as_isize()
    }
}

/// sys_execve - replace current process with new program
fn sys_execve(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    serial::write_string("sys_execve: not fully implemented\n");
    Errno::ENOSYS.as_isize()
}

/// sys_open - open a file (stub - returns error)
fn sys_open(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    serial::write_string("sys_open: not implemented\n");
    Errno::ENOSYS.as_isize()
}

/// sys_close - close a file descriptor (stub - returns success)
fn sys_close(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}

/// sys_getppid - get parent process ID
fn sys_getppid(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    if let Some(current) = SCHEDULER.current_task() {
        let parent_id: usize = current.lock().parent_id;
        parent_id as SyscallResult
    } else {
        Errno::Ok.as_isize()
    }
}

/// sys_setuid - set user ID (stub - returns success)
fn sys_setuid(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}

/// sys_setgid - set group ID (stub - returns success)
fn sys_setgid(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}
