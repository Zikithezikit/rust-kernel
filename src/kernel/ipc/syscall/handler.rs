//! System call handler
//!
//! Handles system call interrupts (int 0x80).
//! Uses Linux-style syscall table and calling convention:
//! - syscall number in eax/rax
//! - arguments in ebx/rdi, ecx/rsi, edx/rdx, esi/r10, edi/r8, ebp/r9
//! - return value in eax/rax

use crate::arch::x86::regs::PtRegs;
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

pub use crate::ipc::syscall::numbers::{
    Errno, FileDescriptor, SyscallFn, SyscallNumber, SyscallResult, NR_SYSCALLS,
};

/// Default time slice for forked tasks (in ticks)
const FORK_TIME_SLICE: usize = 10;

/// Syscall table - maps syscall numbers to functions
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

/// System call handler with registers
///
/// Called from arch/x86/interrupts.rs
pub unsafe fn handle_syscall_with_regs(regs: &mut PtRegs) -> SyscallResult {
    let syscall_num = regs.rax as usize;
    let arg1 = regs.rdi as usize;
    let arg2 = regs.rsi as usize;
    let arg3 = regs.rdx as usize;
    let arg4 = regs.r10 as usize;
    let arg5 = regs.r8 as usize;
    let arg6 = regs.r9 as usize;

    // Check if coming from user mode
    if (regs.cs & 3) == 3 {
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
            let result: SyscallResult = fn_ptr(regs, arg1, arg2, arg3, arg4, arg5, arg6);
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
    _regs: &mut PtRegs,
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
    _regs: &mut PtRegs,
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
    _regs: &mut PtRegs,
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
    _regs: &mut PtRegs,
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

/// sys_getppid - get parent process ID
fn sys_getppid(
    _regs: &mut PtRegs,
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

/// sys_fork - create a child process
fn sys_fork(
    regs: &mut PtRegs,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    if let Some(parent_task_arc) = SCHEDULER.current_task() {
        let (parent_id, parent_cr3, parent_stack_top) = {
            let p = parent_task_arc.lock();
            (p.id, p.cr3, p.kernel_stack_top)
        };

        let new_id: usize = match TASK_ID_ALLOCATOR.alloc() {
            Some(id) => id,
            None => return Errno::EAGAIN.as_isize(),
        };

        // 1. Clone address space
        use crate::mm::page_tables::copy_address_space;
        use x86_64::PhysAddr;

        let child_cr3_phys = unsafe { copy_address_space(PhysAddr::new(parent_cr3 as u64)) }
            .expect("Failed to clone address space");

        // 2. Allocate new kernel stack
        let child_stack_top: usize = {
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

        // 3. Copy kernel stack content
        // In our assembly handler, regs points to the start of PtRegs on the stack
        let regs_ptr = regs as *const PtRegs as usize;
        let stack_used = parent_stack_top - regs_ptr;
        let child_regs_ptr = child_stack_top - stack_used;

        unsafe {
            core::ptr::copy_nonoverlapping(
                regs_ptr as *const u8,
                child_regs_ptr as *mut u8,
                stack_used,
            );
        }

        // 4. Adjust child registers
        let child_regs = unsafe { &mut *(child_regs_ptr as *mut PtRegs) };
        child_regs.rax = 0; // Fork return value for child

        // 5. Prepare child stack for context_switch
        // Our context_switch pops 6 regs, then 'ret'
        // We want 'ret' to jump to syscall_exit_asm
        extern "C" {
            fn syscall_exit_asm();
        }

        let mut child_rsp = child_regs_ptr;

        // Push return address for context_switch
        child_rsp -= 8;
        unsafe {
            *(child_rsp as *mut usize) = syscall_exit_asm as *const () as usize;
        }

        // Push 6 dummy registers for context_switch
        for _ in 0..6 {
            child_rsp -= 8;
            unsafe {
                *(child_rsp as *mut usize) = 0;
            }
        }

        // 6. Create child Task struct
        let child = Arc::new(Mutex::new(Task {
            id: new_id,
            parent_id,
            state: TaskState::Ready,
            kernel_stack: child_rsp,
            kernel_stack_top: child_stack_top,
            stack_size: KERNEL_STACK_SIZE,
            cr3: child_cr3_phys.as_u64() as usize,
            instruction_pointer: child_regs.rip as usize,
            time_slice: FORK_TIME_SLICE,
            time_slice_max: FORK_TIME_SLICE,
            ticks_run: 0,
            name: "forked",
        }));

        // 7. Add child to scheduler
        SCHEDULER.add_task(child);

        serial::write_string(&format!(
            "sys_fork: parent={}, child={}\n",
            parent_id, new_id
        ));

        new_id as SyscallResult // Parent returns child PID
    } else {
        Errno::ENOMEM.as_isize()
    }
}

/// sys_execve - replace current process with new program
fn sys_execve(
    _regs: &mut PtRegs,
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
    _regs: &mut PtRegs,
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
    _regs: &mut PtRegs,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}

/// sys_getuid - get current user ID
fn sys_getuid(
    _regs: &mut PtRegs,
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
    _regs: &mut PtRegs,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}

/// sys_setuid - set user ID (stub - returns success)
fn sys_setuid(
    _regs: &mut PtRegs,
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
    _regs: &mut PtRegs,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    Errno::Ok.as_isize()
}
