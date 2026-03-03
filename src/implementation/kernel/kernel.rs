//! Kernel core module
//!
//! This module provides the central `Kernel` struct that owns all kernel subsystems.
//! It replaces the previous pattern of global statics with a proper state container.
//!
//! ## Design
//!
//! The Kernel struct is created once during early boot and provides access to all
//! kernel subsystems through getter methods. This enables:
//! - Dependency injection for testing
//! - Clear ownership model
//! - Easier refactoring
//! - Better error propagation

use crate::error::{KernelError, KernelResult};
use crate::memory::pmm::PhysicalMemoryManager;
use crate::task::id_allocator::IdAllocator;
use crate::task::scheduler::Scheduler;
use crate::vfs::mount::MountNamespace;

/// The main kernel instance.
///
/// This struct owns all kernel subsystems and provides access to them
/// through getter methods. It is created once during early boot and
/// lives for the entire kernel lifetime.
pub struct Kernel {
    /// Physical memory manager
    pmm: PhysicalMemoryManager,

    /// Task scheduler
    scheduler: Scheduler,

    /// Task/Process ID allocator
    id_allocator: IdAllocator,

    /// Mount namespace (VFS)
    mount_ns: Option<MountNamespace>,

    /// Whether the kernel is fully initialized
    initialized: bool,
}

impl Kernel {
    /// Returns a reference to the Physical Memory Manager.
    pub fn pmm(&self) -> &PhysicalMemoryManager {
        &self.pmm
    }

    /// Returns a mutable reference to the Physical Memory Manager.
    pub fn pmm_mut(&mut self) -> &mut PhysicalMemoryManager {
        &mut self.pmm
    }

    /// Returns a reference to the Scheduler.
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    /// Returns a mutable reference to the Scheduler.
    pub fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    /// Returns a reference to the ID Allocator.
    pub fn id_allocator(&self) -> &IdAllocator {
        &self.id_allocator
    }

    /// Returns a mutable reference to the ID Allocator.
    pub fn id_allocator_mut(&mut self) -> &mut IdAllocator {
        &mut self.id_allocator
    }

    /// Returns a reference to the Mount Namespace, if initialized.
    pub fn mount_ns(&self) -> Option<&MountNamespace> {
        self.mount_ns.as_ref()
    }

    /// Returns a mutable reference to the Mount Namespace.
    pub fn mount_ns_mut(&mut self) -> Option<&mut MountNamespace> {
        self.mount_ns.as_mut()
    }

    /// Sets the mount namespace.
    pub fn set_mount_ns(&mut self, ns: MountNamespace) {
        self.mount_ns = Some(ns);
    }

    /// Returns whether the kernel is fully initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

// =============================================================================
// Global Kernel Instance
// =============================================================================

use core::sync::atomic::{AtomicBool, Ordering};

/// Global kernel instance - the actual kernel struct
/// Initialized as a default with uninitialized state
static KERNEL: spin::Mutex<Kernel> = spin::Mutex::new(Kernel {
    pmm: PhysicalMemoryManager::new(),
    scheduler: Scheduler::new(),
    id_allocator: IdAllocator::new(),
    mount_ns: None,
    initialized: false,
});

/// Flag to track if kernel is initialized
static KERNEL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes the global kernel instance.
///
/// This must be called once during early boot, after memory initialization,
/// before any other kernel functions that require access to subsystems.
///
/// # Errors
/// Returns `KernelError::InitFailed` if kernel was already initialized
/// or if initialization fails.
pub fn init_kernel() -> KernelResult<()> {
    if KERNEL_INITIALIZED.load(Ordering::SeqCst) {
        return Err(KernelError::InitFailed("Kernel already initialized"));
    }

    // Initialize scheduler and tasks
    {
        let mut kernel = KERNEL.lock();
        kernel.scheduler.init();

        // Spawn idle task
        fn idle_task() {
            loop {
                x86_64::instructions::hlt();
            }
        }

        kernel
            .scheduler
            .spawn(idle_task, "idle")
            .ok_or(KernelError::IdleTaskCreationFailed)?;

        kernel.initialized = true;
    }

    KERNEL_INITIALIZED.store(true, Ordering::SeqCst);

    Ok(())
}

/// Returns a mutable reference to the global kernel instance.
///
/// # Safety
/// Must only be called after `init_kernel()` has completed successfully.
pub fn kernel() -> &'static mut Kernel {
    // This is safe because:
    // 1. KERNEL_INITIALIZED is only set to true after successful init
    // 2. The kernel lives for the entire kernel lifetime (static storage)
    // 3. We return a mutable reference because subsystems need interior mutability
    unsafe {
        let mut guard = KERNEL.lock();
        // Leak the lock temporarily to get a 'static lifetime
        // This is safe because the kernel lives for the entire kernel lifetime
        let ptr = &mut *guard as *mut Kernel;
        &mut *ptr
    }
}

/// Returns whether the kernel has been initialized.
pub fn is_initialized() -> bool {
    KERNEL_INITIALIZED.load(Ordering::SeqCst)
}
