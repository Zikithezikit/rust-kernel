#![allow(dead_code)]

use crate::drivers::vga;
use crate::memory::vmm;
use crate::syscall::handler::handle_syscall;
use crate::task::scheduler::SCHEDULER;
use pc_keyboard::{layouts::Us104Key, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use spin::Once;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static IDT: Once<InterruptDescriptorTable> = Once::new();
static KEYBOARD: Mutex<Option<Keyboard<Us104Key, ScancodeSet1>>> = Mutex::new(None);

const MASTER_CONTROL: u16 = 0x20;
const MASTER_DATA: u16 = 0x21;
const SLAVE_CONTROL: u16 = 0xA0;
const SLAVE_DATA: u16 = 0xA1;

const KEYBOARD_DATA_PORT: u16 = 0x60;

const IRQ_BASE_MASTER: u8 = 32;
const IRQ_BASE_SLAVE: u8 = 40;

const PIC_INIT: u8 = 0x11;
const PIC_ICW4_8086: u8 = 0x01;

const SLAVE_IRQ_LINE: u8 = 0x04;
const SLAVE_IDENTITY: u8 = 0x02;

const END_OF_INTERRUPT: u8 = 0x20;

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;

const PIT_MODE_RATE_GENERATOR: u8 = 0x36;

const SYSCALL_VECTOR: u8 = 0x80; // Matches SyscallNumber::SysExit = 0 but int 0x80 is separate

#[derive(Clone, Copy)]
pub enum IrqNumber {
    SystemTimer,
    Keyboard,
    InterruptController,
    SerialPort2,
    SerialPort1,
    ParallelPort2,
    FloppyDisk,
    ParallelPort1,
    RealTimeClock,
    ACPI,
    Reserved1,
    Reserved2,
    Mouse,
    FPU,
    PrimaryATA,
    SecondaryATA,
}

pub fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
    idt.debug.set_handler_fn(debug_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt[IRQ_BASE_MASTER].set_handler_fn(timer_interrupt_handler);
    idt[IRQ_BASE_MASTER + 1].set_handler_fn(keyboard_interrupt_handler);

    // Syscall handler (int 0x80)
    // Must use set_handler_fn since 0x80 is not a CPU exception
    idt[128].set_handler_fn(syscall_interrupt_handler);

    IDT.call_once(|| idt).load();

    *KEYBOARD.lock() = Some(Keyboard::new(
        ScancodeSet1::default(),
        Us104Key,
        HandleControl::Ignore,
    ));
}

pub unsafe fn init_pic() {
    let mut master_control = Port::new(MASTER_CONTROL);
    let mut master_data = Port::new(MASTER_DATA);
    let mut slave_control = Port::new(SLAVE_CONTROL);
    let mut slave_data = Port::new(SLAVE_DATA);

    master_control.write(PIC_INIT);
    slave_control.write(PIC_INIT);

    master_data.write(IRQ_BASE_MASTER);
    slave_data.write(IRQ_BASE_SLAVE);

    master_data.write(SLAVE_IRQ_LINE);
    slave_data.write(SLAVE_IDENTITY);

    master_data.write(PIC_ICW4_8086);
    slave_data.write(PIC_ICW4_8086);

    master_data.write(0x00);
    slave_data.write(0x00);
}

pub unsafe fn init_pit() {
    let mut command = Port::new(PIT_COMMAND);
    let mut channel0 = Port::new(PIT_CHANNEL_0);

    command.write(PIT_MODE_RATE_GENERATOR);

    let divisor: u16 = (1193180u32 / 100) as u16;
    channel0.write((divisor & 0xFF) as u8);
    channel0.write(((divisor >> 8) & 0xFF) as u8);
}

extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
    vga::println("EXCEPTION: DEBUG");
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    vmm::page_fault_handler(&stack_frame, error_code.bits());
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use crate::syscall::handler::increment_ticks;

    increment_ticks();

    // TODO: Enable preemptive scheduling once context switching is properly implemented
    // For now, just increment ticks without switching
    // SCHEDULER.preemptive_tick();

    pic_end_of_interrupt(IrqNumber::SystemTimer as u8);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::interrupts::without_interrupts;

    unsafe {
        without_interrupts(|| {
            if let Some(ref mut keyboard) = KEYBOARD.lock().as_mut() {
                let mut port = Port::new(KEYBOARD_DATA_PORT);
                let scancode: u8 = port.read();
                if let Ok(Some(_event)) = keyboard.add_byte(scancode) {
                    if let Some(key) = keyboard.process_keyevent(_event) {
                        match key {
                            pc_keyboard::DecodedKey::Unicode(character) => {
                                vga::print_char(character as u8)
                            }
                            pc_keyboard::DecodedKey::RawKey(_key) => {}
                        }
                    }
                }
            }
            pic_end_of_interrupt(IrqNumber::Keyboard as u8);
        });
    }
}

extern "x86-interrupt" fn syscall_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use crate::syscall::handler::handle_syscall;

    let result = unsafe { handle_syscall(&_stack_frame) };

    unsafe {
        core::arch::asm!("mov rax, {}", in(reg) result);
    }
}

fn pic_end_of_interrupt(irq: u8) {
    let mut master = Port::new(MASTER_CONTROL);
    let mut slave = Port::new(SLAVE_CONTROL);

    if irq >= 8 {
        unsafe { slave.write(END_OF_INTERRUPT) };
    }
    unsafe { master.write(END_OF_INTERRUPT) };
}
