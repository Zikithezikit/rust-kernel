//! Kernel error types
//!
//! This module provides a unified error handling system for the entire kernel.
//! All modules should use `KernelError` and `KernelResult` instead of creating
//! their own error types.
//!
//! ## Design Principles
//!
//! - All errors are represented as an enum for exhaustive matching
//! - Errors carry meaningful context for debugging
//! - The error type implements `Display` for user-friendly messages
//! - Errors can be converted from module-specific errors using `From`

use core::fmt;

/// Unified error type for the entire kernel.
///
/// This enum consolidates all error conditions from different subsystems
/// into a single type, making error handling consistent across the codebase.
///
/// # Example
///
/// ```ignore
/// fn some_function() -> KernelResult<SomeType> {
///     // Instead of creating custom errors:
///     // return Err(MyCustomError::Something);
///
///     // Use KernelError:
///     return Err(KernelError::NotFound);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    // ========================================================================
    // Memory subsystem errors
    // ========================================================================
    /// Out of memory - no more pages available
    OutOfMemory,

    /// Invalid memory address
    InvalidAddress,

    /// Page fault occurred
    PageFault,

    /// Invalid page alignment
    InvalidAlignment,

    /// Memory manager not initialized
    MemoryNotInitialized,

    // ========================================================================
    // VFS (Virtual File System) errors
    // ========================================================================
    /// File or directory not found
    NotFound,

    /// Permission denied
    PermissionDenied,

    /// File or directory already exists
    Exists,

    /// Directory not empty
    NotEmpty,

    /// Path component is a directory when a file was expected
    IsDirectory,

    /// Path component is a file when a directory was expected
    NotADirectory,

    /// File too large
    FileTooLarge,

    /// No space left on device
    NoSpaceLeft,

    /// Invalid filename
    InvalidFilename,

    /// Invalid seek position
    InvalidSeek,

    /// Operation not supported by this filesystem/inode
    OperationNotSupported,

    /// Generic I/O error
    IoError,

    /// Resource is busy
    Busy,

    /// Too many open files
    TooManyOpenFiles,

    /// Invalid file descriptor
    BadFileDescriptor,

    // ========================================================================
    // Task/Process management errors
    // ========================================================================
    /// Task not found
    TaskNotFound,

    /// Task already running
    TaskAlreadyRunning,

    /// Task is in wrong state for operation
    InvalidTaskState,

    /// Scheduler is full
    SchedulerFull,

    /// Failed to allocate task stack
    StackAllocationFailed,

    /// Failed to create idle task
    IdleTaskCreationFailed,

    // ========================================================================
    // System call errors
    // ========================================================================
    /// Invalid syscall number
    InvalidSyscall,

    /// Syscall not implemented
    SyscallNotImplemented,

    /// Argument out of range
    InvalidArgument,

    // ========================================================================
    // Initialization errors
    // ========================================================================
    /// Initialization failed with a reason string
    InitFailed(&'static str),

    /// Driver initialization failed
    DriverFailed(&'static str),

    /// Memory initialization failed
    MemoryInitFailed(&'static str),

    /// Task initialization failed
    TaskInitFailed(&'static str),

    // ========================================================================
    // Architecture/Hardware errors
    // ========================================================================
    /// TSS initialization failed
    TssInitFailed,

    /// GDT initialization failed
    GdtInitFailed,

    /// IDT initialization failed
    IdtInitFailed,

    /// Interrupt handling error
    InterruptError,

    // ========================================================================
    // Generic/Other errors
    // ========================================================================
    /// Operation not implemented yet
    NotImplemented,

    /// Invalid state - operation called at wrong time
    InvalidState,

    /// Something went wrong (generic fallback)
    Unknown,

    // ========================================================================
    // Device/Driver errors
    // ========================================================================
    /// Device not found
    DeviceNotFound,

    /// Device error
    DeviceError(&'static str),

    /// Device not ready
    DeviceNotReady,

    /// Buffer too small
    BufferTooSmall,
}

impl KernelError {
    /// Returns a human-readable description of the error category.
    pub fn category(&self) -> &'static str {
        match self {
            // Memory
            Self::OutOfMemory => "OutOfMemory",
            Self::InvalidAddress => "InvalidAddress",
            Self::PageFault => "PageFault",
            Self::InvalidAlignment => "InvalidAlignment",
            Self::MemoryNotInitialized => "MemoryNotInitialized",

            // VFS
            Self::NotFound => "NotFound",
            Self::PermissionDenied => "PermissionDenied",
            Self::Exists => "Exists",
            Self::NotEmpty => "NotEmpty",
            Self::IsDirectory => "IsDirectory",
            Self::NotADirectory => "NotADirectory",
            Self::FileTooLarge => "FileTooLarge",
            Self::NoSpaceLeft => "NoSpaceLeft",
            Self::InvalidFilename => "InvalidFilename",
            Self::InvalidSeek => "InvalidSeek",
            Self::OperationNotSupported => "OperationNotSupported",
            Self::IoError => "IoError",
            Self::Busy => "Busy",
            Self::TooManyOpenFiles => "TooManyOpenFiles",
            Self::BadFileDescriptor => "BadFileDescriptor",

            // Task
            Self::TaskNotFound => "TaskNotFound",
            Self::TaskAlreadyRunning => "TaskAlreadyRunning",
            Self::InvalidTaskState => "InvalidTaskState",
            Self::SchedulerFull => "SchedulerFull",
            Self::StackAllocationFailed => "StackAllocationFailed",
            Self::IdleTaskCreationFailed => "IdleTaskCreationFailed",

            // Syscall
            Self::InvalidSyscall => "InvalidSyscall",
            Self::SyscallNotImplemented => "SyscallNotImplemented",
            Self::InvalidArgument => "InvalidArgument",

            // Init
            Self::InitFailed(_) => "InitFailed",
            Self::DriverFailed(_) => "DriverFailed",
            Self::MemoryInitFailed(_) => "MemoryInitFailed",
            Self::TaskInitFailed(_) => "TaskInitFailed",

            // Arch
            Self::TssInitFailed => "TssInitFailed",
            Self::GdtInitFailed => "GdtInitFailed",
            Self::IdtInitFailed => "IdtInitFailed",
            Self::InterruptError => "InterruptError",

            // Generic
            Self::NotImplemented => "NotImplemented",
            Self::InvalidState => "InvalidState",
            Self::Unknown => "Unknown",

            // Device
            Self::DeviceNotFound => "DeviceNotFound",
            Self::DeviceError(_) => "DeviceError",
            Self::DeviceNotReady => "DeviceNotReady",
            Self::BufferTooSmall => "BufferTooSmall",
        }
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Memory errors
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::InvalidAddress => write!(f, "Invalid memory address"),
            Self::PageFault => write!(f, "Page fault occurred"),
            Self::InvalidAlignment => write!(f, "Invalid memory alignment"),
            Self::MemoryNotInitialized => write!(f, "Memory manager not initialized"),

            // VFS errors
            Self::NotFound => write!(f, "File or directory not found"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::Exists => write!(f, "File or directory already exists"),
            Self::NotEmpty => write!(f, "Directory not empty"),
            Self::IsDirectory => write!(f, "Is a directory"),
            Self::NotADirectory => write!(f, "Not a directory"),
            Self::FileTooLarge => write!(f, "File too large"),
            Self::NoSpaceLeft => write!(f, "No space left on device"),
            Self::InvalidFilename => write!(f, "Invalid filename"),
            Self::InvalidSeek => write!(f, "Invalid seek position"),
            Self::OperationNotSupported => write!(f, "Operation not supported"),
            Self::IoError => write!(f, "I/O error"),
            Self::Busy => write!(f, "Resource busy"),
            Self::TooManyOpenFiles => write!(f, "Too many open files"),
            Self::BadFileDescriptor => write!(f, "Bad file descriptor"),

            // Task errors
            Self::TaskNotFound => write!(f, "Task not found"),
            Self::TaskAlreadyRunning => write!(f, "Task already running"),
            Self::InvalidTaskState => write!(f, "Invalid task state"),
            Self::SchedulerFull => write!(f, "Scheduler is full"),
            Self::StackAllocationFailed => write!(f, "Failed to allocate task stack"),
            Self::IdleTaskCreationFailed => write!(f, "Failed to create idle task"),

            // Syscall errors
            Self::InvalidSyscall => write!(f, "Invalid syscall number"),
            Self::SyscallNotImplemented => write!(f, "Syscall not implemented"),
            Self::InvalidArgument => write!(f, "Invalid argument"),

            // Init errors
            Self::InitFailed(msg) => write!(f, "Initialization failed: {}", msg),
            Self::DriverFailed(msg) => write!(f, "Driver failed: {}", msg),
            Self::MemoryInitFailed(msg) => write!(f, "Memory init failed: {}", msg),
            Self::TaskInitFailed(msg) => write!(f, "Task init failed: {}", msg),

            // Arch errors
            Self::TssInitFailed => write!(f, "TSS initialization failed"),
            Self::GdtInitFailed => write!(f, "GDT initialization failed"),
            Self::IdtInitFailed => write!(f, "IDT initialization failed"),
            Self::InterruptError => write!(f, "Interrupt handling error"),

            // Generic errors
            Self::NotImplemented => write!(f, "Operation not implemented"),
            Self::InvalidState => write!(f, "Invalid state"),
            Self::Unknown => write!(f, "Unknown error"),

            // Device errors
            Self::DeviceNotFound => write!(f, "Device not found"),
            Self::DeviceError(msg) => write!(f, "Device error: {}", msg),
            Self::DeviceNotReady => write!(f, "Device not ready"),
            Self::BufferTooSmall => write!(f, "Buffer too small"),
        }
    }
}

impl core::error::Error for KernelError {
    fn description(&self) -> &str {
        // Simple description for error chaining
        match self {
            Self::OutOfMemory => "out of memory",
            Self::NotFound => "not found",
            Self::PermissionDenied => "permission denied",
            _ => "kernel error",
        }
    }
}

/// Result type alias for kernel operations.
///
/// All kernel functions that can fail should return `KernelResult<T>` instead
/// of creating custom Result types.
///
/// # Example
///
/// ```ignore
/// fn read_file(path: &str) -> KernelResult<Vec<u8>> {
///     // Function body that may return errors
///     Err(KernelError::NotFound)
/// }
/// ```
pub type KernelResult<T> = Result<T, KernelError>;

// =============================================================================
// From implementations for converting from module-specific errors
// =============================================================================

/// Temporary: VFS error kinds before full migration
/// These variants map to KernelError - used during transition period
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsErrorKind {
    NotFound,
    PermissionDenied,
    Exists,
    NotEmpty,
    IsDirectory,
    NotADirectory,
    FileTooLarge,
    NoSpaceLeft,
    InvalidFilename,
    InvalidSeek,
    OperationNotSupported,
    IoError,
    Busy,
}

impl From<VfsErrorKind> for KernelError {
    fn from(err: VfsErrorKind) -> Self {
        match err {
            VfsErrorKind::NotFound => KernelError::NotFound,
            VfsErrorKind::PermissionDenied => KernelError::PermissionDenied,
            VfsErrorKind::Exists => KernelError::Exists,
            VfsErrorKind::NotEmpty => KernelError::NotEmpty,
            VfsErrorKind::IsDirectory => KernelError::IsDirectory,
            VfsErrorKind::NotADirectory => KernelError::NotADirectory,
            VfsErrorKind::FileTooLarge => KernelError::FileTooLarge,
            VfsErrorKind::NoSpaceLeft => KernelError::NoSpaceLeft,
            VfsErrorKind::InvalidFilename => KernelError::InvalidFilename,
            VfsErrorKind::InvalidSeek => KernelError::InvalidSeek,
            VfsErrorKind::OperationNotSupported => KernelError::OperationNotSupported,
            VfsErrorKind::IoError => KernelError::IoError,
            VfsErrorKind::Busy => KernelError::Busy,
        }
    }
}

/// Converts a result with a legacy VfsError to KernelResult
#[inline]
pub fn vfs_err_to_kernel<T>(res: Result<T, crate::fs::vfs::inode::VfsError>) -> KernelResult<T> {
    res.map_err(|e| match e {
        crate::fs::vfs::inode::VfsError::NotFound => KernelError::NotFound,
        crate::fs::vfs::inode::VfsError::PermissionDenied => KernelError::PermissionDenied,
        crate::fs::vfs::inode::VfsError::Exists => KernelError::Exists,
        crate::fs::vfs::inode::VfsError::NotEmpty => KernelError::NotEmpty,
        crate::fs::vfs::inode::VfsError::IsDirectory => KernelError::IsDirectory,
        crate::fs::vfs::inode::VfsError::NotADirectory => KernelError::NotADirectory,
        crate::fs::vfs::inode::VfsError::FileTooLarge => KernelError::FileTooLarge,
        crate::fs::vfs::inode::VfsError::NoSpaceLeft => KernelError::NoSpaceLeft,
        crate::fs::vfs::inode::VfsError::InvalidFilename => KernelError::InvalidFilename,
        crate::fs::vfs::inode::VfsError::InvalidSeek => KernelError::InvalidSeek,
        crate::fs::vfs::inode::VfsError::OperationNotSupported => {
            KernelError::OperationNotSupported
        }
        crate::fs::vfs::inode::VfsError::IoError => KernelError::IoError,
        crate::fs::vfs::inode::VfsError::Busy => KernelError::Busy,
    })
}
