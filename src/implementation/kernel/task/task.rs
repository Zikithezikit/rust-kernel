//! Task structures and state management
//!
//! Provides the Task struct (similar to Linux task_struct) and TaskState enum
//! for managing kernel tasks/threads.

use alloc::sync::Arc;
use spin::Mutex;

/// Size of kernel stack in bytes (8KB - two pages)
pub const KERNEL_STACK_SIZE: usize = 8192;

/// Task/Thread state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Ready,
    Blocked,
    Zombie,
    New,
}

/// Task ID type
pub type TaskId = usize;

/// Task control block - similar to Linux task_struct
///
/// Contains all information needed to manage a single task/thread:
/// - Execution state (registers, stack pointer)
/// - Scheduling information (state, priority, timeslice)
/// - Resources (kernel stack, memory)
/// - Parent/child relationships
pub struct Task {
    pub id: TaskId,
    pub parent_id: TaskId,
    pub state: TaskState,
    pub kernel_stack: usize,
    pub stack_size: usize,
    pub instruction_pointer: usize,
    pub time_slice: usize,
    pub time_slice_max: usize,
    pub ticks_run: u64,
    pub name: &'static str,
}

impl Task {
    /// Creates a new task with a freshly allocated kernel stack
    ///
    /// # Arguments
    /// * `id` - Unique task identifier
    /// * `entry` - Function pointer to start executing
    /// * `name` - Task name for debugging
    ///
    /// # Safety
    /// - The kernel stack must be properly set up
    /// - The entry function should never return
    pub unsafe fn new(id: TaskId, entry: fn(), name: &'static str) -> Option<Arc<Mutex<Task>>> {
        let stack = Self::allocate_kernel_stack()?;

        let task = Task {
            id,
            parent_id: 0,
            state: TaskState::New,
            kernel_stack: stack,
            stack_size: KERNEL_STACK_SIZE,
            instruction_pointer: entry as usize,
            time_slice: 0,
            time_slice_max: DEFAULT_TIME_SLICE,
            ticks_run: 0,
            name,
        };

        Some(Arc::new(Mutex::new(task)))
    }

    /// Allocates a kernel stack from physical memory
    ///
    /// Returns the top of the stack (highest address).
    /// Stack grows downward, so we'll put the stack pointer at the top.
    ///
    /// # Returns
    /// - Some(top_of_stack) - Highest address in allocated stack
    /// - None if allocation failed
    fn allocate_kernel_stack() -> Option<usize> {
        use crate::memory::pmm::PMM;

        let pages_needed = KERNEL_STACK_SIZE / crate::memory::pmm::PAGE_SIZE;
        let phys_addr = PMM.allocate_pages(pages_needed)?;

        Some(phys_addr + KERNEL_STACK_SIZE)
    }

    /// Deallocates the kernel stack
    pub fn free_kernel_stack(&self) {
        use crate::memory::pmm::PMM;

        let pages_needed = KERNEL_STACK_SIZE / crate::memory::pmm::PAGE_SIZE;
        let stack_bottom = self.kernel_stack - KERNEL_STACK_SIZE;
        PMM.deallocate_pages(stack_bottom, pages_needed);
    }

    /// Sets the task to Ready state
    pub fn set_ready(&mut self) {
        self.state = TaskState::Ready;
    }

    /// Sets the task to Running state
    pub fn set_running(&mut self) {
        self.state = TaskState::Running;
    }

    /// Sets the task to Blocked state
    pub fn set_blocked(&mut self) {
        self.state = TaskState::Blocked;
    }

    /// Checks if task is runnable (Ready or Running)
    pub fn is_runnable(&self) -> bool {
        matches!(self.state, TaskState::Ready | TaskState::Running)
    }
}

/// Default time slice for round-robin scheduling (in timer ticks)
/// 10 ticks at ~100Hz timer = ~100ms
pub const DEFAULT_TIME_SLICE: usize = 10;
