//! Task management initialization module
//!
//! Handles scheduler and task initialization.

use crate::task::scheduler::SCHEDULER;

#[derive(Debug)]
pub enum TaskError {
    SchedulerInitFailed,
    IdleTaskSpawnFailed,
}

impl core::fmt::Display for TaskError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TaskError::SchedulerInitFailed => write!(f, "Scheduler initialization failed"),
            TaskError::IdleTaskSpawnFailed => write!(f, "Failed to spawn idle task"),
        }
    }
}

pub fn init_scheduler() -> Result<(), TaskError> {
    SCHEDULER.init();
    Ok(())
}

pub fn spawn_idle_task() -> Result<(), TaskError> {
    fn idle_task() {
        loop {
            x86_64::instructions::hlt();
        }
    }
    SCHEDULER.spawn(idle_task, "idle");
    Ok(())
}

pub fn init() -> Result<(), TaskError> {
    init_scheduler()?;
    spawn_idle_task()
}
