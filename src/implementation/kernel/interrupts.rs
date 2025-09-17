use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use spin::Once;
use spin::Mutex;
use x86_64::instructions::port::Port;
use pc_keyboard::{Keyboard, layouts::Us104Key, ScancodeSet1, HandleControl};
use crate::std_lib::vga;

static IDT: Once<InterruptDescriptorTable> = Once::new();
static KEYBOARD: Mutex<Option<Keyboard<Us104Key, ScancodeSet1>>> = Mutex::new(None);

pub fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
    idt.debug.set_handler_fn(debug_handler);
    idt[32].set_handler_fn(timer_interrupt_handler);
    idt[33].set_handler_fn(keyboard_interrupt_handler);
    IDT.call_once(|| idt).load();

    *KEYBOARD.lock() = Some(Keyboard::new(ScancodeSet1::default(), Us104Key, HandleControl::Ignore));
}

pub unsafe fn init_pic() {
    let mut master = Port::new(0x20u16);
    let mut master_data = Port::new(0x21u16);
    let mut slave = Port::new(0xA0u16);
    let mut slave_data = Port::new(0xA1u16);

    // Start initialization (ICW1)
    master.write(0x11u8);
    slave.write(0x11u8);

    // Set vector offsets (ICW2)
    master_data.write(0x20u8); // Master offset
    slave_data.write(0x28u8);  // Slave offset

    // Tell master about slave (ICW3)
    master_data.write(0x04u8); // IRQ2 has slave
    slave_data.write(0x02u8);  // Slave identity

    // ICW4: 8086 mode
    master_data.write(0x01u8);
    slave_data.write(0x01u8);

    // Mask all IRQs (just in case)
    master_data.write(0x0u8);
    slave_data.write(0x0u8);
    
}

extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
    vga::println("EXCEPTION: DEBUG");
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    pic_end_of_interrupt(0); // IRQ0
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
                            pc_keyboard::DecodedKey::Unicode(character) => vga::print_char(character as u8),
                            pc_keyboard::DecodedKey::RawKey(_key) => {} 
                        }
                    }
                }
            }
            pic_end_of_interrupt(1);
        });
    }
}


fn pic_end_of_interrupt(irq: u8) {
    let mut master = Port::new(0x20);
    let mut slave = Port::new(0xA0);

    if irq >= 8 { unsafe { slave.write(0x20u8) }; }
    unsafe { master.write(0x20u8) };
}
