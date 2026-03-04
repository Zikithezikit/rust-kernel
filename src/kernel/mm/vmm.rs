//! Virtual Memory Manager (VMM)
//!
//! This module implements virtual memory management for the kernel.
//! It provides page fault handling and virtual address space management.
//!
//! ## Overview
//!
//! The VMM handles:
//! - Page fault handling (demand paging)
//! - Virtual address space management
//! - Page table manipulation
//!
//! ## Page Fault Handling
//!
//! When a page fault occurs, the handler checks the fault reason:
//! - **Present (P) bit = 0**: Page not present - attempt to allocate
//! - **Write (W) bit = 1**: Write access - may be copy-on-write
//! - **User (U) bit = 1**: User mode access
//! - **Reserved bit = 1**: Reserved bit violation (should not happen)

use alloc::format;
use spin::Mutex;

use crate::drivers::serial;
use crate::mm::pmm::PMM;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::InterruptStackFrame;

/// Page size (4KB)
const PAGE_SIZE: usize = 4096;

/// Page fault error bits
mod fault_bits {
    pub const PRESENT: u64 = 1 << 0; // Page protection violation
    pub const WRITE: u64 = 1 << 1; // Write access
    pub const USER: u64 = 1 << 2; // User mode access
    pub const RESERVED: u64 = 1 << 3; // Reserved bit violation
    pub const INSTRUCTION: u64 = 1 << 4; // Instruction fetch
}

/// Global VMM state (protected by mutex for mutability)
static VMM: Mutex<VmmState> = Mutex::new(VmmState::new());

/// VMM state structure
#[derive(Debug, Clone, Copy)]
pub struct VmmState {
    /// Number of page faults handled
    page_faults: usize,
    /// Number of pages allocated on demand
    pages_allocated: usize,
}

impl VmmState {
    /// Create new VMM state
    pub const fn new() -> Self {
        VmmState {
            page_faults: 0,
            pages_allocated: 0,
        }
    }

    /// Increment page fault counter
    pub fn count_fault(&mut self) {
        self.page_faults += 1;
    }

    /// Increment pages allocated counter
    pub fn count_page_alloc(&mut self) {
        self.pages_allocated += 1;
    }

    /// Get stats
    pub fn get_stats(&self) -> (usize, usize) {
        (self.page_faults, self.pages_allocated)
    }
}

/// Page fault handler
///
/// This is called when a page fault exception occurs (interrupt 14).
/// It analyzes the fault and attempts to handle it appropriately.
pub fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) {
    // Get the faulting address
    let fault_addr = match Cr2::read() {
        Ok(addr) => addr.as_u64() as usize,
        Err(_) => {
            serial::write_string("VMM: Could not read fault address\n");
            halt_on_fault(stack_frame, error_code, 0);
            return;
        }
    };

    // Count this fault
    VMM.lock().count_fault();

    // Analyze the error code
    let present = (error_code & fault_bits::PRESENT) != 0;
    let write = (error_code & fault_bits::WRITE) != 0;
    let user = (error_code & fault_bits::USER) != 0;
    let reserved = (error_code & fault_bits::RESERVED) != 0;

    // Log the fault (limited output to avoid flooding serial)
    let vmm_stats = *VMM.lock();
    if vmm_stats.page_faults <= 10 || vmm_stats.page_faults % 1000 == 0 {
        serial::write_string(&format!(
            "VMM: Page fault at addr=0x{:x}, code=0x{:x} (P={}, W={}, U={})\n",
            fault_addr, error_code, present, write, user
        ));
    }

    // Handle reserved bit violation - this is a serious error
    if reserved {
        serial::write_string("VMM: Reserved bit violation - fatal!\n");
        halt_on_fault(stack_frame, error_code, fault_addr);
        return;
    }

    // Handle protection violation (present page but access denied)
    if present {
        // This is typically a write to a read-only page
        // For now, treat as fatal
        serial::write_string("VMM: Protection violation - page is present but access denied\n");
        halt_on_fault(stack_frame, error_code, fault_addr);
        return;
    }

    // Handle non-present page - try to allocate a new page
    // This is demand paging
    if !present {
        handle_demand_page(fault_addr, write, user);
        return;
    }

    // Unknown fault type
    serial::write_string("VMM: Unknown fault type\n");
    halt_on_fault(stack_frame, error_code, fault_addr);
}

/// Handle demand paging - allocate a new page for a faulting address
///
/// # Arguments
/// * `fault_addr` - The virtual address that caused the fault
/// * `write` - Whether this was a write access
/// * `user` - Whether this was a user mode access
fn handle_demand_page(fault_addr: usize, _write: bool, user: bool) {
    // For now, we only handle kernel page faults
    // User space faults would require more complex address space management
    if user {
        serial::write_string(&format!(
            "VMM: User mode page fault at 0x{:x} - not implemented yet\n",
            fault_addr
        ));
        // For now, just halt on user page faults
        return;
    }

    // Check if address is in reasonable range
    // Kernel is typically mapped at high addresses
    if fault_addr < 0x1000 {
        // Null pointer dereference
        serial::write_string("VMM: Null pointer dereference!\n");
        return;
    }

    // Try to allocate a page
    match PMM.allocate_page() {
        Some(phys_addr) => {
            VMM.lock().count_page_alloc();

            // For now, we just log that we'd map this page
            // Full implementation would involve updating page tables
            let pages_allocated = VMM.lock().pages_allocated;
            if pages_allocated <= 10 || pages_allocated % 1000 == 0 {
                serial::write_string(&format!(
                    "VMM: Allocated page for 0x{:x} -> physical 0x{:x}\n",
                    fault_addr, phys_addr
                ));
            }

            // Note: In a full implementation, we would:
            // 1. Walk the page tables
            // 2. Find or create the page table entries
            // 3. Map the physical page to the virtual address
            // 4. Set appropriate permissions (read/write/execute)
        }
        None => {
            serial::write_string("VMM: Out of memory - cannot allocate page!\n");
        }
    }
}

/// Halts on unrecoverable page fault
fn halt_on_fault(stack_frame: &InterruptStackFrame, error_code: u64, fault_addr: usize) {
    serial::write_string("======================================\n");
    serial::write_string("PAGE FAULT - KERNEL PANIC\n");
    serial::write_string("======================================\n");

    serial::write_string(&format!("Fault address: 0x{:x}\n", fault_addr));
    serial::write_string(&format!("Error code: 0x{:x}\n", error_code));
    serial::write_string(&format!(
        "Instruction pointer: 0x{:x}\n",
        stack_frame.instruction_pointer.as_u64()
    ));
    serial::write_string(&format!(
        "Stack pointer: 0x{:x}\n",
        stack_frame.stack_pointer.as_u64()
    ));

    if error_code & 1 != 0 {
        serial::write_string("Cause: Protection violation (page present)\n");
    } else {
        serial::write_string("Cause: Page not present\n");
    }

    if error_code & 2 != 0 {
        serial::write_string("Access: Write\n");
    } else {
        serial::write_string("Access: Read\n");
    }

    if error_code & 4 != 0 {
        serial::write_string("Mode: User\n");
    } else {
        serial::write_string("Mode: Kernel\n");
    }

    // Print VMM stats
    let stats = *VMM.lock();
    serial::write_string(&format!("Total page faults: {}\n", stats.page_faults));
    serial::write_string(&format!("Pages allocated: {}\n", stats.pages_allocated));

    // Halt the system
    loop {
        x86_64::instructions::hlt();
    }
}

/// Get VMM statistics
pub fn get_stats() -> (usize, usize) {
    let stats = VMM.lock();
    (stats.page_faults, stats.pages_allocated)
}

/// Initialize the VMM
pub fn init() {
    serial::write_string("VMM initialized\n");
}
