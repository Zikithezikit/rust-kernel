#![allow(dead_code)]

use crate::std_lib::vga;
use pc_keyboard::{layouts::Us104Key, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use spin::Once;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: Once<InterruptDescriptorTable> = Once::new();
static KEYBOARD: Mutex<Option<Keyboard<Us104Key, ScancodeSet1>>> = Mutex::new(None);

const MASTER_CONTROL: u16 = 0x20;
const MASTER_DATA: u16 = 0x21;

const SLAVE_CONTROL: u16 = 0xA0;
const SLAVE_DATA: u16 = 0xA1;

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
    idt[32].set_handler_fn(timer_interrupt_handler);
    idt[33].set_handler_fn(keyboard_interrupt_handler);
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

    // Start initialization (ICW1)
    master_control.write(0x11u8);
    slave_control.write(0x11u8);

    // Set vector offsets (ICW2)
    master_data.write(MASTER_CONTROL as u8); // Master offset
    slave_data.write(SLAVE_CONTROL as u8); // Slave offset

    // Tell master about slave (ICW3)
    master_data.write(0x04u8); // IRQ2 has slave
    slave_data.write(0x02u8); // Slave identity

    // ICW4: 8086 mode
    master_data.write(0x01u8);
    slave_data.write(0x01u8);

    // Mask all IRQs (just in case)
    master_data.write(0x0u8);
    slave_data.write(0x0u8);
}

extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
    vga::println("EXCEPTION: DEBUG");
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    pic_end_of_interrupt(IrqNumber::SystemTimer as u8);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::interrupts::without_interrupts;

    unsafe {
        without_interrupts(|| {
            if let Some(ref mut keyboard) = KEYBOARD.lock().as_mut() {
                let mut port = Port::new(0x60);
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

fn pic_end_of_interrupt(irq: u8) {
    let mut master = Port::new(MASTER_CONTROL);
    let mut slave = Port::new(SLAVE_CONTROL);
    const END_OF_INTERRUPT: u8 = 0x20;

    if irq >= 8 {
        unsafe { slave.write(END_OF_INTERRUPT) };
    }
    unsafe { master.write(END_OF_INTERRUPT) };
}
