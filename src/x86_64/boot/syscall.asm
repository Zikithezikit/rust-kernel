[bits 64]
global syscall_entry_asm
global syscall_exit_asm
extern handle_syscall_rust

section .text

; Syscall entry point for int 0x80
syscall_entry_asm:
    ; CPU pushes: ss, rsp, rflags, cs, rip
    
    ; Push dummy error code for consistency with PtRegs
    push 0
    
    ; Push all general purpose registers in order
    ; Matches PtRegs layout in src/kernel/arch/x86/regs.rs
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15
    
    ; Pass pointer to stack frame as first argument to Rust handler
    mov rdi, rsp
    
    ; Ensure stack is 16-byte aligned for FFI
    sub rsp, 8
    call handle_syscall_rust
    add rsp, 8
    
syscall_exit_asm:
    ; Restore all registers
    ; If handle_syscall_rust modified regs->rax, it will be restored here
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax
    
    ; Remove dummy error code
    add rsp, 8
    
    ; Return to user mode
    iretq
