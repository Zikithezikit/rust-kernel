//! System call numbers
//!
//! Defines the system call numbers used for kernel-user communication.
//! Follows Linux syscall numbering style (though actual numbers differ).

use core::fmt;

/// System call return type
pub type SyscallResult = isize;

/// System call function signature
pub type SyscallFn = fn(usize, usize, usize, usize, usize, usize) -> SyscallResult;

/// Maximum number of syscalls
pub const NR_syscalls: usize = 128;

/// Standard file descriptors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDescriptor {
    Stdin = 0,
    Stdout = 1,
    Stderr = 2,
}

impl FileDescriptor {
    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(FileDescriptor::Stdin),
            1 => Some(FileDescriptor::Stdout),
            2 => Some(FileDescriptor::Stderr),
            _ => None,
        }
    }

    pub fn as_usize(self) -> usize {
        self as usize
    }
}

/// Error codes (Linux-style errno)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Errno {
    /// Success
    Ok = 0,
    /// Operation not permitted
    EPERM = 1,
    /// No such file or directory
    ENOENT = 2,
    /// No such process
    ESRCH = 3,
    /// Interrupted system call
    EINTR = 4,
    /// I/O error
    EIO = 5,
    /// No such device or address
    ENXIO = 6,
    /// Argument list too long
    E2BIG = 7,
    /// Exec format error
    ENOEXEC = 8,
    /// Bad file number
    EBADF = 9,
    /// No child processes
    ECHILD = 10,
    /// Try again
    EAGAIN = 11,
    /// Out of memory
    ENOMEM = 12,
    /// Permission denied
    EACCES = 13,
    /// Bad address
    EFAULT = 14,
    /// Device or resource busy
    EBUSY = 16,
    /// File exists
    EEXIST = 17,
    /// Cross-device link
    EXDEV = 18,
    /// No such device
    ENODEV = 19,
    /// Not a directory
    ENOTDIR = 20,
    /// Is a directory
    EISDIR = 21,
    /// Invalid argument
    EINVAL = 22,
    /// File table overflow
    ENFILE = 23,
    /// Too many open files
    EMFILE = 24,
    /// Not a typewriter
    ENOTTY = 25,
    /// No space left on device
    ENOSPC = 28,
    /// Read-only file system
    EROFS = 30,
    /// Too many links
    EMLINK = 31,
    /// Broken pipe
    EPIPE = 32,
    /// Math argument out of domain of func
    EDOM = 33,
    /// Math result not representable
    ERANGE = 34,
    /// Resource temporarily unavailable
    EAGAINAlt = 35,
    /// Operation now in progress
    EINPROGRESS = 36,
    /// Operation already in progress
    EALREADY = 37,
    /// Function not implemented
    ENOSYS = 38,
    /// Directory not empty
    ENOTEMPTY = 39,
    /// Too many levels of symbolic links
    ELOOP = 40,
}

impl Errno {
    pub fn as_isize(self) -> SyscallResult {
        -(self as isize)
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Errno::Ok => "OK",
            Errno::EPERM => "EPERM",
            Errno::ENOENT => "ENOENT",
            Errno::ESRCH => "ESRCH",
            Errno::EINTR => "EINTR",
            Errno::EIO => "EIO",
            Errno::ENXIO => "ENXIO",
            Errno::E2BIG => "E2BIG",
            Errno::ENOEXEC => "ENOEXEC",
            Errno::EBADF => "EBADF",
            Errno::ECHILD => "ECHILD",
            Errno::EAGAIN => "EAGAIN",
            Errno::ENOMEM => "ENOMEM",
            Errno::EACCES => "EACCES",
            Errno::EFAULT => "EFAULT",
            Errno::EBUSY => "EBUSY",
            Errno::EEXIST => "EEXIST",
            Errno::EXDEV => "EXDEV",
            Errno::ENODEV => "ENODEV",
            Errno::ENOTDIR => "ENOTDIR",
            Errno::EISDIR => "EISDIR",
            Errno::EINVAL => "EINVAL",
            Errno::ENFILE => "ENFILE",
            Errno::EMFILE => "EMFILE",
            Errno::ENOTTY => "ENOTTY",
            Errno::ENOSPC => "ENOSPC",
            Errno::EROFS => "EROFS",
            Errno::EMLINK => "EMLINK",
            Errno::EPIPE => "EPIPE",
            Errno::EDOM => "EDOM",
            Errno::ERANGE => "ERANGE",
            Errno::EAGAINAlt => "EAGAIN",
            Errno::EINPROGRESS => "EINPROGRESS",
            Errno::EALREADY => "EALREADY",
            Errno::ENOSYS => "ENOSYS",
            Errno::ENOTEMPTY => "ENOTEMPTY",
            Errno::ELOOP => "ELOOP",
        };
        write!(f, "{}", name)
    }
}

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
    /// Converts a u32 to SyscallNumber (safe conversion)
    #[inline]
    pub fn from_u32(n: u32) -> Option<SyscallNumber> {
        match n {
            0 => Some(SyscallNumber::SysExit),
            1 => Some(SyscallNumber::SysWrite),
            2 => Some(SyscallNumber::SysRead),
            3 => Some(SyscallNumber::SysOpen),
            4 => Some(SyscallNumber::SysClose),
            5 => Some(SyscallNumber::SysGetPid),
            6 => Some(SyscallNumber::SysGetPpid),
            7 => Some(SyscallNumber::SysFork),
            8 => Some(SyscallNumber::SysExecve),
            9 => Some(SyscallNumber::SysWait4),
            10 => Some(SyscallNumber::SysClone),
            11 => Some(SyscallNumber::SysExitGroup),
            12 => Some(SyscallNumber::SysGetTimeOfDay),
            13 => Some(SyscallNumber::SysNanosleep),
            14 => Some(SyscallNumber::SysTimes),
            15 => Some(SyscallNumber::SysReserved),
            16 => Some(SyscallNumber::SysBrk),
            17 => Some(SyscallNumber::SysMmap),
            18 => Some(SyscallNumber::SysMunmap),
            19 => Some(SyscallNumber::SysMprotect),
            20 => Some(SyscallNumber::SysGetuid),
            21 => Some(SyscallNumber::SysGetgid),
            22 => Some(SyscallNumber::SysSetuid),
            23 => Some(SyscallNumber::SysSetgid),
            _ => None,
        }
    }

    /// Returns the syscall number as usize
    #[inline]
    pub fn as_usize(self) -> usize {
        self as usize
    }
}
