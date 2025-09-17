use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use spin::Once;
use crate::std_lib::vga;

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
    idt.debug.set_handler_fn(debug_handler);
    IDT.call_once(|| idt).load();
}


extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
    vga::println("EXCEPTION: DEBUG");
}
