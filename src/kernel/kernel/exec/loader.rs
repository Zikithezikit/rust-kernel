//! Logic for loading ELF files into address spaces

use super::elf::{Elf64Header, Elf64Phdr, PT_LOAD};
use crate::arch::x86::regs::PtRegs;
use crate::include::consts::{PAGE_SIZE, USER_STACK_PAGES, USER_STACK_TOP};
use crate::include::error::{KernelError, KernelResult};
use crate::kernel_instance::kernel;
use crate::mm::page_tables::map_user_page;
use crate::mm::pmm::PMM;
use alloc::vec::Vec;
use x86_64::structures::paging::{Page, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub fn load_and_run(path: &str, regs: &mut PtRegs) -> KernelResult<()> {
    // 1. Open and read the file
    let mut file_data = Vec::new();
    {
        let ns_guard = kernel().mount_ns_mut().ok_or(KernelError::NotImplemented)?;
        let inode = ns_guard
            .walk_path(path)
            .map_err(|_| KernelError::NotFound)?;
        let inode_lock = inode.lock();

        let size = inode_lock.stat().st_size as usize;
        file_data.resize(size, 0);
        inode_lock
            .read(0, &mut file_data)
            .map_err(|_| KernelError::IoError)?;
    }

    // 2. Parse ELF header
    if file_data.len() < core::mem::size_of::<Elf64Header>() {
        return Err(KernelError::InvalidAddress);
    }

    let header = unsafe { &*(file_data.as_ptr() as *const Elf64Header) };
    if !header.is_valid() {
        return Err(KernelError::InvalidAddress);
    }

    // 3. Create new address space (optional for execve but recommended)
    // For now, we'll just map into the current address space (which should be a new one if called from fork)

    // 4. Map and load segments
    let phdr_size = core::mem::size_of::<Elf64Phdr>();
    for i in 0..header.e_phnum {
        let offset = header.e_phoff as usize + (i as usize * phdr_size);
        let phdr = unsafe { &*(file_data.as_ptr().add(offset) as *const Elf64Phdr) };

        if phdr.p_type == PT_LOAD {
            load_segment(&file_data, phdr)?;
        }
    }

    // 5. Set up user stack
    let stack_top = USER_STACK_TOP;
    let stack_pages = USER_STACK_PAGES;
    for i in 0..stack_pages {
        let virt = stack_top - (i as u64 + 1) * PAGE_SIZE as u64;
        let phys = PMM.allocate_page().ok_or(KernelError::OutOfMemory)?;
        unsafe {
            map_user_page(
                Page::containing_address(VirtAddr::new(virt)),
                PhysFrame::containing_address(PhysAddr::new(phys as u64)),
            )
            .map_err(|_| KernelError::OutOfMemory)?;
        }
    }

    // 6. Update registers for return to user mode
    regs.rip = header.e_entry;
    regs.rsp = stack_top;
    regs.rax = 0; // Return 0 to user

    Ok(())
}

fn load_segment(file_data: &[u8], phdr: &Elf64Phdr) -> KernelResult<()> {
    let start_addr = VirtAddr::new(phdr.p_vaddr);
    let end_addr = VirtAddr::new(phdr.p_vaddr + phdr.p_memsz);

    // Align to page boundaries
    let start_page = Page::<Size4KiB>::containing_address(start_addr);
    let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(end_addr.as_u64() - 1));

    for page in Page::range_inclusive(start_page, end_page) {
        let phys = PMM.allocate_page().ok_or(KernelError::OutOfMemory)?;
        unsafe {
            map_user_page(
                page,
                PhysFrame::containing_address(PhysAddr::new(phys as u64)),
            )
            .map_err(|_| KernelError::OutOfMemory)?;

            // Clear page
            core::ptr::write_bytes(phys as *mut u8, 0, PAGE_SIZE);
        }

        // Copy data from file
        let page_start = page.start_address().as_u64();
        let page_end = page_start + PAGE_SIZE as u64;

        let copy_start = core::cmp::max(page_start, phdr.p_vaddr);
        let copy_end = core::cmp::min(page_end, phdr.p_vaddr + phdr.p_filesz);

        if copy_start < copy_end {
            let offset_in_file = phdr.p_offset + (copy_start - phdr.p_vaddr);
            let offset_in_page = copy_start - page_start;
            let len = copy_end - copy_start;

            unsafe {
                core::ptr::copy_nonoverlapping(
                    file_data.as_ptr().add(offset_in_file as usize),
                    (phys as *mut u8).add(offset_in_page as usize),
                    len as usize,
                );
            }
        }
    }

    Ok(())
}
