; Context switching assembly routines
;
; These functions handle saving and restoring CPU state during task switches.
; The task switch works by:
; 1. Pushing all registers onto the kernel stack
; 2. Saving the stack pointer to the current task's struct
; 3. Loading the new task's stack pointer
; 4. Restoring registers from the new stack

global context_switch
extern current_task_ptr

section .text
bits 64

; void context_switch(usize new_task_rsp)
; Switch from current task to new task
; Input: RDI = new task's stack pointer (RSP)
; Output: Returns to caller with new task's context
context_switch:
    ; Save all general purpose registers (callee-saved: rbx, rbp, r12-r15)
    
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    
    ; Save current RSP to current task's kernel_stack field
    ; current_task_ptr now points directly to the Task struct
    mov rax, [rel current_task_ptr]
    test rax, rax
    jz .no_current_task
    
    ; kernel_stack is at offset 24 in Task struct (#[repr(C)])
    mov [rax + 24], rsp
    
.no_current_task:
    ; Load new task's stack pointer
    mov rsp, rdi
    
    ; Restore all general purpose registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    
    ; Return to the new task
    ret

; Initial task switch - sets up the stack for the first task
; This is called when starting the very first user task
global task_startup
extern first_task_rsp

section .text
bits 64

task_startup:
    ; At this point, RSP points to the new task's kernel stack
    ; The stack should have the return address pushed
    
    ; Restore initial state
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    
    ; Enable interrupts (they were disabled during boot)
    sti
    
    ; Jump to the task's entry point
    ; The entry point was stored where ret would return to
    ret

section .note.GNU-stack noalloc noexec nowrite progbits
