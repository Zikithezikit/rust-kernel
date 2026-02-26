//! Task management module
//!
//! This module provides process/thread management functionality:
//! - Task structures (similar to Linux task_struct)
//! - Task state management
//! - Kernel stack allocation
//! - Context switching
//! - Scheduler

pub mod id_allocator;
pub mod scheduler;
pub mod switch;
pub mod task;
