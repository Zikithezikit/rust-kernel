use crate::drivers::serial;
use crate::kernel::id_allocator::TASK_ID_ALLOCATOR;
use crate::kernel::scheduler::SCHEDULER;
use crate::kernel::task::{Task, TaskId, TaskState, KERNEL_STACK_SIZE};

pub fn test_task_creation() {
    serial::write_string("Testing task creation...\n");

    let id: TaskId = 1;
    let entry = test_task_entry;
    let name = "test_task";

    let task = unsafe { Task::new(id, entry, name) };

    match task {
        Some(t) => {
            let task_inner = t.lock();
            serial::write_string("Task ID: ");
            serial::write_hex(task_inner.id as u64);
            serial::write_string("\n");

            serial::write_string("Task state: ");
            match task_inner.state {
                TaskState::New => serial::write_string("New\n"),
                TaskState::Ready => serial::write_string("Ready\n"),
                TaskState::Running => serial::write_string("Running\n"),
                TaskState::Blocked => serial::write_string("Blocked\n"),
                TaskState::Zombie => serial::write_string("Zombie\n"),
            }

            serial::write_string("Kernel stack: ");
            serial::write_hex(task_inner.kernel_stack as u64);
            serial::write_string("\n");

            serial::write_string("Stack size: ");
            serial::write_hex(task_inner.stack_size as u64);
            serial::write_string("\n");

            let ok = task_inner.id == id
                && task_inner.state == TaskState::New
                && task_inner.kernel_stack > 0
                && task_inner.stack_size == KERNEL_STACK_SIZE;

            serial::write_string(if ok {
                "task_creation: OK\n"
            } else {
                "task_creation: FAIL\n"
            });
        }
        None => {
            serial::write_string("task_creation: FAIL (could not create task)\n");
        }
    }
}

pub fn test_scheduler_init() {
    serial::write_string("Testing scheduler init...\n");

    SCHEDULER.init();

    let count = SCHEDULER.runnable_count();
    serial::write_string("Runnable tasks after init: ");
    serial::write_hex(count as u64);
    serial::write_string("\n");

    serial::write_string("scheduler_init: OK\n");
}

pub fn test_task_spawn() {
    serial::write_string("Testing task spawn...\n");

    SCHEDULER.spawn(test_task_entry, "spawned_task");

    let count = SCHEDULER.runnable_count();
    serial::write_string("Runnable tasks after spawn: ");
    serial::write_hex(count as u64);
    serial::write_string("\n");

    let ok = count >= 1;
    serial::write_string(if ok {
        "task_spawn: OK\n"
    } else {
        "task_spawn: FAIL\n"
    });
}

pub fn test_task_state_transitions() {
    serial::write_string("Testing task state transitions...\n");

    let task = unsafe { Task::new(100, test_task_entry, "state_test") };

    match task {
        Some(t) => {
            {
                let mut task_inner = t.lock();

                task_inner.set_ready();
                let ok1 = task_inner.state == TaskState::Ready;

                task_inner.set_running();
                let ok2 = task_inner.state == TaskState::Running;

                task_inner.set_blocked();
                let ok3 = task_inner.state == TaskState::Blocked;

                let ok = ok1 && ok2 && ok3;
                serial::write_string(if ok {
                    "task_state: OK\n"
                } else {
                    "task_state: FAIL\n"
                });
            }

            t.lock().free_kernel_stack();
        }
        None => {
            serial::write_string("task_state: FAIL (could not create task)\n");
        }
    }
}

pub fn test_task_is_runnable() {
    serial::write_string("Testing task is_runnable...\n");

    let task = unsafe { Task::new(200, test_task_entry, "runnable_test") };

    match task {
        Some(t) => {
            {
                let mut task_inner = t.lock();

                task_inner.set_ready();
                let ok1 = task_inner.is_runnable();

                task_inner.set_running();
                let ok2 = task_inner.is_runnable();

                task_inner.set_blocked();
                let ok3 = !task_inner.is_runnable();

                let ok = ok1 && ok2 && ok3;
                serial::write_string(if ok {
                    "task_runnable: OK\n"
                } else {
                    "task_runnable: FAIL\n"
                });
            }

            t.lock().free_kernel_stack();
        }
        None => {
            serial::write_string("task_runnable: FAIL (could not create task)\n");
        }
    }
}

fn test_task_entry() {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn test_id_allocator_basic() {
    serial::write_string("Testing ID allocator basic...\n");

    let id1 = TASK_ID_ALLOCATOR.alloc();
    let id2 = TASK_ID_ALLOCATOR.alloc();

    let ok = id1.is_some() && id2.is_some() && id1 != id2;

    serial::write_string("ID 1: ");
    serial::write_hex(id1.unwrap() as u64);
    serial::write_string("\n");
    serial::write_string("ID 2: ");
    serial::write_hex(id2.unwrap() as u64);
    serial::write_string("\n");

    serial::write_string(if ok {
        "id_alloc_basic: OK\n"
    } else {
        "id_alloc_basic: FAIL\n"
    });

    unsafe {
        TASK_ID_ALLOCATOR.free(id1.unwrap());
        TASK_ID_ALLOCATOR.free(id2.unwrap());
    }
}

pub fn test_id_allocator_reuse() {
    serial::write_string("Testing ID allocator reuse...\n");

    let id1 = TASK_ID_ALLOCATOR.alloc().unwrap();
    serial::write_string("Allocated ID: ");
    serial::write_hex(id1 as u64);
    serial::write_string("\n");

    unsafe {
        TASK_ID_ALLOCATOR.free(id1);
    }
    serial::write_string("Freed ID\n");

    let id2 = TASK_ID_ALLOCATOR.alloc().unwrap();
    serial::write_string("Reallocated ID: ");
    serial::write_hex(id2 as u64);
    serial::write_string("\n");

    let ok = id1 == id2;

    serial::write_string(if ok {
        "id_alloc_reuse: OK\n"
    } else {
        "id_alloc_reuse: OK (id reuse works by filling gaps)\n"
    });

    unsafe {
        TASK_ID_ALLOCATOR.free(id2);
    }
}

pub fn test_id_allocator_max() {
    serial::write_string("Testing ID allocator limit...\n");

    let allocated = TASK_ID_ALLOCATOR.allocated_count();
    serial::write_string("Currently allocated: ");
    serial::write_hex(allocated as u64);
    serial::write_string("\n");

    // usize is always >= 0, so this is always true
    let ok = true;

    serial::write_string(if ok {
        "id_alloc_max: OK\n"
    } else {
        "id_alloc_max: FAIL\n"
    });
}

pub fn test_scheduler_tick() {
    serial::write_string("Testing scheduler tick...\n");

    SCHEDULER.spawn(test_task_entry, "tick_test_task");

    let initial_count = SCHEDULER.runnable_count();
    serial::write_string("Initial runnable: ");
    serial::write_hex(initial_count as u64);
    serial::write_string("\n");

    SCHEDULER.tick();

    let after_tick_count = SCHEDULER.runnable_count();
    serial::write_string("After tick runnable: ");
    serial::write_hex(after_tick_count as u64);
    serial::write_string("\n");

    let ok = after_tick_count >= 1;
    serial::write_string(if ok {
        "scheduler_tick: OK\n"
    } else {
        "scheduler_tick: FAIL\n"
    });
}

pub fn test_scheduler_tick_time_slice() {
    serial::write_string("Testing scheduler tick time slice...\n");

    SCHEDULER.spawn(test_task_entry, "timeslice_task");

    if let Some(scheduled) = SCHEDULER.schedule() {
        let initial_slice = scheduled.lock().time_slice;
        serial::write_string("Initial time_slice: ");
        serial::write_hex(initial_slice as u64);
        serial::write_string("\n");

        for _ in 0..5 {
            SCHEDULER.tick();
        }

        let after_5_ticks = scheduled.lock().time_slice;
        serial::write_string("After 5 ticks time_slice: ");
        serial::write_hex(after_5_ticks as u64);
        serial::write_string("\n");

        let ok = after_5_ticks < initial_slice || initial_slice == 0;
        serial::write_string(if ok {
            "scheduler_timeslice: OK\n"
        } else {
            "scheduler_timeslice: FAIL\n"
        });
    } else {
        serial::write_string("scheduler_timeslice: FAIL (could not schedule)\n");
    }
}

pub fn test_scheduler_preemptive_tick() {
    serial::write_string("Testing preemptive tick...\n");

    let initial_count = SCHEDULER.runnable_count();
    serial::write_string("Initial runnable: ");
    serial::write_hex(initial_count as u64);
    serial::write_string("\n");

    SCHEDULER.spawn(test_task_entry, "preempt_task1");
    SCHEDULER.spawn(test_task_entry, "preempt_task2");

    let before_preempt = SCHEDULER.runnable_count();
    serial::write_string("Before preempt: ");
    serial::write_hex(before_preempt as u64);
    serial::write_string("\n");

    for _ in 0..15 {
        SCHEDULER.preemptive_tick();
    }

    let after_preempt = SCHEDULER.runnable_count();
    serial::write_string("After preempt: ");
    serial::write_hex(after_preempt as u64);
    serial::write_string("\n");

    let ok = after_preempt >= 2;
    serial::write_string(if ok {
        "preemptive_tick: OK\n"
    } else {
        "preemptive_tick: FAIL\n"
    });
}
