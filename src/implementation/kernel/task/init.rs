//! Task management initialization module
//!
//! Handles scheduler and task initialization.

use crate::error::{KernelError, KernelResult};
use crate::task::scheduler::SCHEDULER;

/// Initializes the scheduler.
///
/// # Errors
/// Returns `KernelError::TaskInitFailed` if scheduler fails to initialize.
pub fn init_scheduler() -> KernelResult<()> {
    SCHEDULER.init();
    Ok(())
}

/// Spawns the idle task.
///
/// The idle task runs when no other tasks are runnable.
/// It simply halts the CPU to save power.
///
/// # Errors
/// Returns `KernelError::IdleTaskCreationFailed` if idle task cannot be spawned.
pub fn spawn_idle_task() -> KernelResult<()> {
    fn idle_task() {
        loop {
            x86_64::instructions::hlt();
        }
    }

    // The spawn method returns Option<TaskId>, convert to KernelResult
    let _task_id = SCHEDULER
        .spawn(idle_task, "idle")
        .ok_or(KernelError::IdleTaskCreationFailed)?;

    Ok(())
}

/// Initializes the task subsystem.
///
/// This function:
/// 1. Initializes the scheduler
/// 2. Spawns the idle task
///
/// # Errors
/// Returns `KernelError::TaskInitFailed` if any step fails.
pub fn init() -> KernelResult<()> {
    init_scheduler()?;
    spawn_idle_task()?;
    Ok(())
}
