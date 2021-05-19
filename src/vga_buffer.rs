use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // We only need 4 bytes, but Rust does not support u4
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        // Foreground color goes into the lower 4 bytes,
        // whereas background color goes into the upper 4.
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // To ensure that field ordering follows the C convention, i.e. in order
struct ScreenChar {
    // Each character is represented by an
    // ASCII byte followed by a color byte
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    // Since we only write and never read to this buffer, the compiler
    // might try to optimise writes away. By marking it as volatile, we
    // tell the compiler that writes here have side effects.
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,      // Where we are in the last row
    color_code: ColorCode,       // Current background and foreground color selected
    buffer: &'static mut Buffer, // The buffer will be valid for the whole program runtime
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII (anything between a
                // space and tilde) and newline
                // https://en.wikipedia.org/wiki/Code_page_437#Character_set
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // We print ■ for unprintable chars
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                // A volatile write so the compiler doesn't optimise
                // this write away
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // Move each character one line up, i.e. the
        // top line gets deleted.
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                // Volatile read so that the compiler will never
                // optimise this away.
                let char = self.buffer.chars[row][col].read();
                // Copy this character one row above where it is
                self.buffer.chars[row - 1][col].write(char);
            }
        }

        // Clear last row
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // Write blank characters to an entire row to clear it
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// So that the writer supports Rust's formatting
// macros, e.g. write! and writeln!
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// Below print macros are largely based on the std's
// definition of print! and println!

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// The macros need to be able to call this from outside
// the module, but we hide it from the generated
// documentation since this can be considered a private
// implementation detail.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
