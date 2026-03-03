use x86_64::instructions::port::Port;

use crate::error::{KernelError, KernelResult};

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
#[allow(dead_code)]
const INTERRUPT_ENABLE_DATA_READY: u8 = 0x01;

const UART_COM1: u16 = 0x3F8; // COM1 base port
const BAUD_DIVISOR_LOW: u8 = 0x01; // Baud divisor low byte (115200 baud)
const BAUD_DIVISOR_HIGH: u8 = 0x00; // Baud divisor high byte
const FLOATING_BUS: u8 = 0xFF; // Value returned by missing UART
const LINE_STATUS_INVALID: u8 = 0x00; // Invalid/empty line status after init

pub unsafe fn init() -> KernelResult<()> {
    let mut line_status: Port<u8> = Port::new(LINE_STATUS);

    // Detect missing UART (floating bus usually returns 0xFF)
    if line_status.read() == FLOATING_BUS {
        return Err(KernelError::DriverFailed("Missing UART"));
    }

    // Disable interrupts
    Port::new(INTERRUPT_ENABLE).write(INTERRUPT_ENABLE_NONE);

    // Enable DLAB to set baud rate
    Port::new(LINE_CONTROL).write(LINE_CONTROL_DLAB);

    // Set divisor to 1 (115200 baud)
    Port::new(DATA_PORT).write(BAUD_DIVISOR_LOW); // low byte
    Port::new(INTERRUPT_ENABLE).write(BAUD_DIVISOR_HIGH); // high byte

    // 8 bits, no parity, 1 stop bit
    Port::new(LINE_CONTROL).write(LINE_CONTROL_8N1);

    // Enable and clear FIFO
    Port::new(FIFO_CONTROL).write(FIFO_CONTROL_ENABLE | FIFO_CONTROL_CLEAR);

    // Set DTR and RTS
    Port::new(MODEM_CONTROL).write(MODEM_CONTROL_DTR | MODEM_CONTROL_RTS);

    // Sanity check: line status should not be zero
    if line_status.read() == LINE_STATUS_INVALID {
        return Err(KernelError::DriverFailed("Serial Line status is 0"));
    }

    Ok(())
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

pub fn write_hex(mut value: u64) {
    let digits = b"0123456789abcdef";
    let mut buf = [0u8; 16];
    let mut i = 16;
    while value != 0 {
        i -= 1;
        buf[i] = digits[(value & 0xf) as usize];
        value >>= 4;
    }
    if i == 16 {
        i -= 1;
        buf[i] = b'0';
    }
    for &byte in &buf[i..] {
        write_byte(byte);
    }
}
