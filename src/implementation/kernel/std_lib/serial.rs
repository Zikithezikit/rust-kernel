use x86_64::instructions::port::Port;

const COM1_PORT: u16 = 0x3F8;

pub unsafe fn init() {
    Port::new(COM1_PORT).write(0x00u8);
    Port::new(COM1_PORT + 1).write(0x00u8);
    Port::new(COM1_PORT + 2).write(0x03u8);
    Port::new(COM1_PORT + 3).write(0x80u8);
    Port::new(COM1_PORT + 0).write(0x01u8);
    Port::new(COM1_PORT + 1).write(0x00u8);
    Port::new(COM1_PORT + 2).write(0x03u8);
    Port::new(COM1_PORT + 3).write(0x03u8);
    Port::new(COM1_PORT + 4).write(0x01u8);
    Port::new(COM1_PORT + 1).write(0x00u8);
}

fn is_transmit_empty() -> bool {
    let mut port: Port<u8> = Port::new(COM1_PORT + 5);
    unsafe { port.read() & 0x20 != 0 }
}

pub fn write_byte(byte: u8) {
    while !is_transmit_empty() {}
    let mut port: Port<u8> = Port::new(COM1_PORT);
    unsafe { port.write(byte) }
}

pub fn write_string(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

pub fn write_char(c: char) {
    write_byte(c as u8);
}
