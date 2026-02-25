// VGA buffer constants and color enum
pub const VGA_BUFFER_ADDRESS: *mut u8 = 0xb8000 as *mut u8;
pub const BUFFER_WIDTH: usize = 80;
pub const BUFFER_HEIGHT: usize = 25;

#[derive(Clone, Copy)]
pub enum ColorCodeVga {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

// Cursor position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cursor {
    Position { col: usize, row: usize },
}

static mut CURSOR: Cursor = Cursor::Position { col: 0, row: 0 };

// Helper to get buffer offset
fn buffer_offset(col: usize, row: usize) -> usize {
    (row * BUFFER_WIDTH + col) * 2
}

// Combine foreground and background color into one byte
pub fn foreground_background_colors(foreground: ColorCodeVga, background: ColorCodeVga) -> u8 {
    (foreground as u8) | ((background as u8) << 4)
}

// Scroll the screen up by one line
pub fn scroll_up() {
    unsafe {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let from_offset = buffer_offset(col, row);
                let to_offset = buffer_offset(col, row - 1);
                *VGA_BUFFER_ADDRESS.add(to_offset) = *VGA_BUFFER_ADDRESS.add(from_offset);
                *VGA_BUFFER_ADDRESS.add(to_offset + 1) = *VGA_BUFFER_ADDRESS.add(from_offset + 1);
            }
        }

        // Clear the last line
        let last_row = BUFFER_HEIGHT - 1;
        for col in 0..BUFFER_WIDTH {
            let offset = buffer_offset(col, last_row);
            *VGA_BUFFER_ADDRESS.add(offset) = b' ';
            *VGA_BUFFER_ADDRESS.add(offset + 1) =
                foreground_background_colors(ColorCodeVga::White, ColorCodeVga::Black);
        }
    }
}

// Print a single character with color at the current cursor
pub fn print_char_with_color(
    byte: u8,
    foreground_color: ColorCodeVga,
    background_color: ColorCodeVga,
) {
    unsafe {
        let mut col;
        let mut row;

        match CURSOR {
            Cursor::Position { col: c, row: r } => {
                col = c;
                row = r;
            }
        }

        match byte {
            b'\n' => {
                col = 0;
                row += 1;
            }
            _ => {
                let offset = buffer_offset(col, row);
                *VGA_BUFFER_ADDRESS.add(offset) = byte;
                *VGA_BUFFER_ADDRESS.add(offset + 1) =
                    foreground_background_colors(foreground_color, background_color);
                col += 1;

                if col >= BUFFER_WIDTH {
                    col = 0;
                    row += 1;
                }
            }
        }

        if row >= BUFFER_HEIGHT {
            scroll_up();
            row = BUFFER_HEIGHT - 1;
        }

        CURSOR = Cursor::Position { col, row };
    }
}

// Print a single character with default colors
pub fn print_char(byte: u8) {
    print_char_with_color(byte, ColorCodeVga::White, ColorCodeVga::Black);
}

// Print a string with color
pub fn print_string_with_color(
    to_print: &str,
    foreground_color: ColorCodeVga,
    background_color: ColorCodeVga,
) {
    for byte in to_print.bytes() {
        print_char_with_color(byte, foreground_color, background_color);
    }
}

// Print a string with default colors
pub fn print_string(to_print: &str) {
    print_string_with_color(to_print, ColorCodeVga::White, ColorCodeVga::Black);
}

// Print a line with default colors
pub fn println(to_print: &str) {
    print_string(to_print);
    print_char_with_color(b'\n', ColorCodeVga::White, ColorCodeVga::Black);
}

// Print a line with color
pub fn println_with_color(
    to_print: &str,
    foreground_color: ColorCodeVga,
    background_color: ColorCodeVga,
) {
    print_string_with_color(to_print, foreground_color, background_color);
    print_char_with_color(b'\n', foreground_color, background_color);
}

// Fill the entire screen with a color
pub fn fill_screen(color: ColorCodeVga) {
    for i in 0..(BUFFER_WIDTH * BUFFER_HEIGHT) {
        let offset = i * 2;
        unsafe {
            *VGA_BUFFER_ADDRESS.add(offset) = b' ';
            *VGA_BUFFER_ADDRESS.add(offset + 1) = color as u8;
        }
    }
}

// Clear the screen to black
pub fn clear_screen() {
    fill_screen(ColorCodeVga::Black);
}

pub fn print(s: &str) {
    print_string(s);
}

pub fn println_hex(value: u64) {
    let digits = b"0123456789abcdef";
    let mut buf = [0u8; 16];
    let mut i = 16;
    let mut v = value;
    while v != 0 {
        i -= 1;
        buf[i] = digits[(v & 0xf) as usize];
        v >>= 4;
    }
    if i == 16 {
        i -= 1;
        buf[i] = b'0';
    }
    for &byte in &buf[i..] {
        print_char(byte);
    }
    print_char(b'\n');
}
