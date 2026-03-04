//! Kernel core module
//!
//! This module contains core kernel functionality similar to Linux kernel/:
//! - Task management (task_struct)
//! - Scheduler
//! - Context switching
//! - Process creation (fork/exec)
//! - PID allocation

pub mod id_allocator;
pub mod scheduler;
pub mod switch;
pub mod task;
