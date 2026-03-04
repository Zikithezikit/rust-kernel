//! User-space syscall wrappers
//!
//! Provides wrappers for making system calls via int 0x80.
//! These are intended to be used from user-space code (or kernel code)
//! to invoke kernel services.

use crate::ipc::syscall::numbers::SyscallNumber;

/// Make a system call with 0 arguments
#[inline]
pub unsafe fn syscall0(nr: SyscallNumber) -> usize {
    let ret: usize;
    core::arch::asm!(
        "int $$0x80",
        in("rax") nr as usize,
        lateout("rax") ret,
    );
    ret
}

/// Make a system call with 1 argument
#[inline]
pub unsafe fn syscall1(nr: SyscallNumber, a1: usize) -> usize {
    let ret: usize;
    core::arch::asm!(
        "int $$0x80",
        in("rax") nr as usize,
        in("rdi") a1,
        lateout("rax") ret,
    );
    ret
}

/// Make a system call with 2 arguments
#[inline]
pub unsafe fn syscall2(nr: SyscallNumber, a1: usize, a2: usize) -> usize {
    let ret: usize;
    core::arch::asm!(
        "int $$0x80",
        in("rax") nr as usize,
        in("rdi") a1,
        in("rsi") a2,
        lateout("rax") ret,
    );
    ret
}

/// Make a system call with 3 arguments
#[inline]
pub unsafe fn syscall3(nr: SyscallNumber, a1: usize, a2: usize, a3: usize) -> usize {
    let ret: usize;
    core::arch::asm!(
        "int $$0x80",
        in("rax") nr as usize,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        lateout("rax") ret,
    );
    ret
}

/// Make a system call with 4 arguments
#[inline]
pub unsafe fn syscall4(nr: SyscallNumber, a1: usize, a2: usize, a3: usize, a4: usize) -> usize {
    let ret: usize;
    core::arch::asm!(
        "int $$0x80",
        in("rax") nr as usize,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        lateout("rax") ret,
    );
    ret
}

/// Make a system call with 5 arguments
#[inline]
pub unsafe fn syscall5(
    nr: SyscallNumber,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> usize {
    let ret: usize;
    core::arch::asm!(
        "int $$0x80",
        in("rax") nr as usize,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        in("r8") a5,
        lateout("rax") ret,
    );
    ret
}

/// Make a system call with 6 arguments
#[inline]
pub unsafe fn syscall6(
    nr: SyscallNumber,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
) -> usize {
    let ret: usize;
    core::arch::asm!(
        "int $$0x80",
        in("rax") nr as usize,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        in("r8") a5,
        in("r9") a6,
        lateout("rax") ret,
    );
    ret
}
