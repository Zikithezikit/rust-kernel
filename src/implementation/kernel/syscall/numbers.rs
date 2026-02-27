//! System call numbers
//!
//! Defines the system call numbers used for kernel-user communication.
//! Follows Linux syscall numbering style (though actual numbers differ).

use core::fmt;

/// System call return type
pub type SyscallResult = isize;

/// System call function signature (Linux-style)
pub type SyscallFn = fn(usize, usize, usize, usize, usize, usize) -> SyscallResult;

/// Maximum number of syscalls
pub const NR_syscalls: usize = 128;

/// System call numbers - follows Linux convention
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SyscallNumber {
    /// Exit the current task
    /// sys_exit(exit_code: i32)
    SysExit = 0,

    /// Write to file descriptor
    /// sys_write(fd: usize, buf: *const u8, count: usize)
    SysWrite = 1,

    /// Read from file descriptor
    /// sys_read(fd: usize, buf: *mut u8, count: usize)
    SysRead = 2,

    /// Open file
    /// sys_open(filename: *const i8, flags: i32, mode: i32)
    SysOpen = 3,

    /// Close file descriptor
    /// sys_close(fd: usize)
    SysClose = 4,

    /// Get current process ID
    /// sys_getpid()
    SysGetPid = 5,

    /// Get current parent process ID
    /// sys_getppid()
    SysGetPpid = 6,

    /// Fork the current process
    /// sys_fork()
    SysFork = 7,

    /// Execute a new program
    /// sys_execve(entry: fn())
    SysExecve = 8,

    /// Wait for process
    /// sys_wait4(pid: isize, status: *mut i32, options: usize)
    SysWait4 = 9,

    /// Create a new process
    /// sys_clone(flags: usize)
    SysClone = 10,

    /// Exit with status and dump core
    /// sys_exit_group(status: i32)
    SysExitGroup = 11,

    /// Get current time
    /// sys_gettimeofday(tv: *mut timeval, tz: *mut timezone)
    SysGetTimeOfDay = 12,

    /// Sleep for specified seconds
    /// sys_nanosleep(req: *const timespec, rem: *mut timespec)
    SysNanosleep = 13,

    /// Get process times
    /// sys_times(tbuf: *mut tms)
    SysTimes = 14,

    /// Unimplemented / reserved
    SysReserved = 15,

    /// Brk - change data segment size
    /// sys_brk(addr: usize)
    SysBrk = 16,

    /// Mmap - map memory
    /// sys_mmap(addr: usize, len: usize, prot: usize, flags: usize, fd: i32, offset: usize)
    SysMmap = 17,

    /// Munmap - unmap memory
    /// sys_munmap(addr: usize, len: usize)
    SysMunmap = 18,

    /// Mprotect - set memory protection
    /// sys_mprotect(addr: usize, len: usize, prot: usize)
    SysMprotect = 19,

    /// Get current UID
    /// sys_getuid()
    SysGetuid = 20,

    /// Get current GID
    /// sys_getgid()
    SysGetgid = 21,

    /// Set UID
    /// sys_setuid(uid: usize)
    SysSetuid = 22,

    /// Set GID
    /// sys_setgid(gid: usize)
    SysSetgid = 23,
}

impl fmt::Display for SyscallNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SyscallNumber::SysExit => "exit",
            SyscallNumber::SysWrite => "write",
            SyscallNumber::SysRead => "read",
            SyscallNumber::SysOpen => "open",
            SyscallNumber::SysClose => "close",
            SyscallNumber::SysGetPid => "getpid",
            SyscallNumber::SysGetPpid => "getppid",
            SyscallNumber::SysFork => "fork",
            SyscallNumber::SysExecve => "execve",
            SyscallNumber::SysWait4 => "wait4",
            SyscallNumber::SysClone => "clone",
            SyscallNumber::SysExitGroup => "exit_group",
            SyscallNumber::SysGetTimeOfDay => "gettimeofday",
            SyscallNumber::SysNanosleep => "nanosleep",
            SyscallNumber::SysTimes => "times",
            SyscallNumber::SysReserved => "reserved",
            SyscallNumber::SysBrk => "brk",
            SyscallNumber::SysMmap => "mmap",
            SyscallNumber::SysMunmap => "munmap",
            SyscallNumber::SysMprotect => "mprotect",
            SyscallNumber::SysGetuid => "getuid",
            SyscallNumber::SysGetgid => "getgid",
            SyscallNumber::SysSetuid => "setuid",
            SyscallNumber::SysSetgid => "setgid",
        };
        write!(f, "{}", name)
    }
}

impl SyscallNumber {
    /// Converts a u32 to SyscallNumber
    #[inline]
    pub fn from_u32(n: u32) -> Option<SyscallNumber> {
        if n <= 23 {
            Some(unsafe { core::mem::transmute(n) })
        } else {
            None
        }
    }

    /// Returns the syscall number as usize
    #[inline]
    pub fn as_usize(self) -> usize {
        self as usize
    }
}
