use x86_64::instructions::port::Port;

const COM1_BASE: u16 = 0x3F8;

const DATA_PORT: u16 = COM1_BASE;
const INTERRUPT_ENABLE: u16 = COM1_BASE + 1;
const FIFO_CONTROL: u16 = COM1_BASE + 2;
const LINE_CONTROL: u16 = COM1_BASE + 3;
const MODEM_CONTROL: u16 = COM1_BASE + 4;
const LINE_STATUS: u16 = COM1_BASE + 5;

const LINE_CONTROL_DLAB: u8 = 0x80;
const LINE_CONTROL_8N1: u8 = 0x03;

const FIFO_CONTROL_ENABLE: u8 = 0x01;
const FIFO_CONTROL_CLEAR: u8 = 0x06;

const MODEM_CONTROL_DTR: u8 = 0x01;
const MODEM_CONTROL_RTS: u8 = 0x02;

const LINE_STATUS_TRANSMIT_EMPTY: u8 = 0x20;

const INTERRUPT_ENABLE_NONE: u8 = 0x00;
const INTERRUPT_ENABLE_DATA_READY: u8 = 0x01;

pub unsafe fn init() {
    Port::new(LINE_CONTROL).write(LINE_CONTROL_DLAB);
    Port::new(DATA_PORT).write(0x01u8);
    Port::new(INTERRUPT_ENABLE).write(INTERRUPT_ENABLE_NONE);

    Port::new(LINE_CONTROL).write(LINE_CONTROL_8N1);
    Port::new(FIFO_CONTROL).write(FIFO_CONTROL_ENABLE | FIFO_CONTROL_CLEAR);
    Port::new(MODEM_CONTROL).write(MODEM_CONTROL_DTR | MODEM_CONTROL_RTS);
    Port::new(INTERRUPT_ENABLE).write(INTERRUPT_ENABLE_NONE);
}

fn is_transmit_empty() -> bool {
    let mut port: Port<u8> = Port::new(LINE_STATUS);
    unsafe { port.read() & LINE_STATUS_TRANSMIT_EMPTY != 0 }
}

pub fn write_byte(byte: u8) {
    while !is_transmit_empty() {}
    let mut port: Port<u8> = Port::new(DATA_PORT);
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
