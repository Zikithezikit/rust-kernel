//! Round-robin scheduler
//!
//! Provides a simple round-robin scheduler for task management.
//! Tasks are kept in a run queue and scheduled in rotation.
//! Uses Linux-style ID allocation with bitmap for reuse.

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;

use super::id_allocator::TASK_ID_ALLOCATOR;
use super::switch::current_task_ptr;
use super::task::{Task, TaskId, TaskState};

/// Global scheduler instance
pub static SCHEDULER: Scheduler = Scheduler::new();

static PREEMPTION_ENABLED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

pub fn set_preemption_enabled(enabled: bool) {
    PREEMPTION_ENABLED.store(enabled, core::sync::atomic::Ordering::SeqCst);
}

pub fn is_preemption_enabled() -> bool {
    PREEMPTION_ENABLED.load(core::sync::atomic::Ordering::SeqCst)
}

/// Round-robin task scheduler
pub struct Scheduler {
    /// Queue of runnable tasks
    run_queue: Mutex<VecDeque<Arc<Mutex<Task>>>>,
    /// Current running task
    current: Mutex<Option<Arc<Mutex<Task>>>>,
    /// Whether scheduler has been initialized
    initialized: Mutex<bool>,
}

impl Scheduler {
    /// Creates a new uninitialized scheduler
    pub const fn new() -> Self {
        Scheduler {
            run_queue: Mutex::new(VecDeque::new()),
            current: Mutex::new(None),
            initialized: Mutex::new(false),
        }
    }

    /// Initializes the scheduler
    ///
    /// Called once during kernel startup.
    pub fn init(&self) {
        let mut init = self.initialized.lock();
        if *init {
            return;
        }
        *init = true;
        drop(init);

        crate::drivers::serial::write_string("Scheduler initialized\n");
    }

    /// Adds a new task to the run queue
    ///
    /// # Arguments
    /// * `task` - The task to add
    pub fn add_task(&self, task: Arc<Mutex<Task>>) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut t = task.lock();
            t.set_ready();
            drop(t);

            self.run_queue.lock().push_back(task);
        });
    }

    /// Gets the next task to run (round-robin)
    ///
    /// # Returns
    /// - Some(task) - Next task to execute
    /// - None if no runnable tasks
    pub fn schedule(&self) -> Option<Arc<Mutex<Task>>> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut queue = self.run_queue.lock();

            if queue.is_empty() {
                return None;
            }

            let task = queue.pop_front()?;
            {
                let mut t = task.lock();
                if t.state == TaskState::Ready {
                    t.set_running();
                }
            }
            *self.current.lock() = Some(task.clone());

            Some(task)
        })
    }

    /// Called when current task's time slice expires
    ///
    /// Moves current task to back of run queue if still runnable.
    pub fn tick(&self) {
        let current = match self.current.lock().take() {
            Some(t) => t,
            None => return,
        };

        let should_reschedule = {
            let mut t = current.lock();
            if t.time_slice > 0 {
                t.time_slice -= 1;
                false
            } else {
                t.time_slice = t.time_slice_max;
                t.ticks_run += 1;

                if t.state == TaskState::Running {
                    t.set_ready();
                    true
                } else {
                    false
                }
            }
        };

        if should_reschedule {
            self.run_queue.lock().push_back(current);
        } else {
            *self.current.lock() = Some(current);
        }
    }

    /// Timer tick with preemptive rescheduling
    ///
    /// Called from timer interrupt. If current task's time slice expired,
    /// switches to next runnable task.
    pub fn preemptive_tick(&self) {
        let current_task_opt = self.current.lock().take();

        if let Some(current) = current_task_opt {
            let should_reschedule = {
                let mut t = current.lock();
                if t.time_slice > 0 {
                    t.time_slice -= 1;
                    false
                } else {
                    t.time_slice = t.time_slice_max;
                    t.ticks_run += 1;

                    if t.state == TaskState::Running {
                        t.set_ready();
                        true
                    } else {
                        false
                    }
                }
            };

            if should_reschedule {
                self.run_queue.lock().push_back(current);

                if let Some(next_task) = self.schedule() {
                    unsafe {
                        let task_ref = next_task.lock();
                        current_task_ptr = &*task_ref as *const Task as usize;
                        let rsp = task_ref.kernel_stack;
                        drop(task_ref);

                        super::switch::context_switch(rsp);
                    }
                }
            } else {
                *self.current.lock() = Some(current);
            }
        } else {
            // No task currently running, try to schedule one
            if let Some(next_task) = self.schedule() {
                unsafe {
                    let task_ref = next_task.lock();
                    current_task_ptr = &*task_ref as *const Task as usize;
                    let rsp = task_ref.kernel_stack;
                    drop(task_ref);

                    super::switch::context_switch(rsp);
                }
            }
        }
    }

    /// Gets the currently running task
    pub fn current_task(&self) -> Option<Arc<Mutex<Task>>> {
        self.current.lock().clone()
    }

    /// Removes a task from the scheduler
    ///
    /// Used when a task exits. Frees the task ID back to the allocator.
    pub fn remove_task(&self, task_id: TaskId) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut queue = self.run_queue.lock();
            queue.retain(|t| t.lock().id != task_id);

            if let Some(ref current) = *self.current.lock() {
                if current.lock().id == task_id {
                    *self.current.lock() = None;
                }
            }

            unsafe {
                TASK_ID_ALLOCATOR.free(task_id);
            }
        });
    }

    /// Creates a new task and adds it to the run queue
    ///
    /// # Arguments
    /// * `entry` - Function to execute
    /// * `name` - Task name for debugging
    ///
    /// # Returns
    /// - Some(task) - The created task
    /// - None if creation failed (no IDs available)
    pub fn spawn(&self, entry: fn(), name: &'static str) -> Option<Arc<Mutex<Task>>> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let id = TASK_ID_ALLOCATOR.alloc()?;

            let task = unsafe { Task::new(id, entry, name)? };
            let task_arc = task.clone();
            self.add_task(task_arc);
            Some(task)
        })
    }

    /// Returns the number of runnable tasks
    pub fn runnable_count(&self) -> usize {
        let queue_len = self.run_queue.lock().len();
        let current_running = if self.current.lock().is_some() { 1 } else { 0 };
        queue_len + current_running
    }

    /// Runs the scheduler loop.
    ///
    /// This is the main entry point for task scheduling. It keeps the kernel alive
    /// by handling timer interrupts and other events.
    pub fn run(&self) -> ! {
        use crate::drivers::serial;

        serial::write_string("Scheduler running...\n");

        // Enable interrupts and wait for timer ticks
        x86_64::instructions::interrupts::enable();

        // Main idle loop - this should never exit
        loop {
            x86_64::instructions::hlt();
        }
    }

    /// Internal schedule method (non-locking version for run loop)
    fn schedule_internal(&self) -> Option<Arc<Mutex<Task>>> {
        let mut queue = self.run_queue.lock();

        if queue.is_empty() {
            return None;
        }

        let task = queue.pop_front()?;
        {
            let mut t = task.lock();
            if t.state == TaskState::Ready {
                t.set_running();
            }
        }
        *self.current.lock() = Some(task.clone());

        Some(task)
    }

    /// Resets the scheduler to a clean state.
    ///
    /// This should be called before running each scheduler test
    /// to avoid leftover tasks, locks, or IDs from previous tests.
    pub fn reset(&self) {
        // Clear the run queue
        let mut runing_queue = self.run_queue.lock();
        runing_queue.clear();
        drop(runing_queue);

        // Clear the current task
        let mut current = self.current.lock();
        *current = None;
        drop(current);

        // Reset the initialized flag (optional, if needed for tests)
        let mut init = self.initialized.lock();
        *init = false;
        drop(init);

        // Reset the task ID allocator
        unsafe { TASK_ID_ALLOCATOR.reset() };

        crate::drivers::serial::write_string("Scheduler reset complete\n");
    }
}
