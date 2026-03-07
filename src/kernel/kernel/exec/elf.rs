//! Minimal ELF64 definitions for the kernel loader

use crate::include::consts::{ELF_MACHINE_X86_64, ELF_TYPE_EXEC};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

pub const PT_LOAD: u32 = 1;

pub const PF_X: u32 = 1;
pub const PF_W: u32 = 2;
pub const PF_R: u32 = 4;

impl Elf64Header {
    pub fn is_valid(&self) -> bool {
        &self.e_ident[0..4] == ELF_MAGIC
            && self.e_type == ELF_TYPE_EXEC
            && self.e_machine == ELF_MACHINE_X86_64
    }
}
