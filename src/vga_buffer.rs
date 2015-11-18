#![macro_use]

use core::cell::Cell;
use core::ptr::Unique;
use core::fmt::{Write, Result};
use spin::Mutex;



const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 25;

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    row_position: 0,
    column_position: 0,
    color_code: Cell::new(ColorCode::new(Color::WHITE, Color::BLACK)),
    buffer: unsafe { Unique::new(0xb8000 as *mut _) } ,
});


macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
            use core::fmt::Write;
            $crate::vga_buffer::WRITER.lock().write_fmt(format_args!($($arg)*)).unwrap();
    });
}

macro_rules! set_color {
    ($front:ident) => ($crate::vga_buffer::switch_color(
        $crate::vga_buffer::Color::$front, 
        $crate::vga_buffer::Color::BLACK)
    );

    (/$back:ident) => ($crate::vga_buffer::switch_color(
        $crate::vga_buffer::Color::WHITE, 
        $crate::vga_buffer::Color::$back)
    );

    ($front:ident/$back:ident) => ($crate::vga_buffer::switch_color(
        $crate::vga_buffer::Color::$front, 
        $crate::vga_buffer::Color::$back)
    )
}

macro_rules! reset_color {
    () => ($crate::vga_buffer::switch_color(
        $crate::vga_buffer::Color::WHITE, 
        $crate::vga_buffer::Color::BLACK)
    )
}


#[repr(u8)]
pub enum Color {
	BLACK      = 0,
    BLUE       = 1,
    GREEN      = 2,
    CYAN       = 3,
    RED        = 4,
    MAGENTA    = 5,
    BROWN      = 6,
    LIGHT_GRAY = 7,
    DARK_GRAY  = 8,
    LIGHT_BLUE = 9,
    LIGHT_GREEN= 10,
    LIGHT_CYAN = 11,
    LIGHT_RED  = 12,
    PINK       = 13,
    YELLOW     = 14,
    WHITE      = 15,
}

#[derive(Clone,Copy)]
pub struct ColorCode(u8);

impl ColorCode {
	pub const fn new(foreground: Color, background: Color) -> ColorCode {
		ColorCode((background as u8) << 4 | (foreground as u8))
	}
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ScreenChar {
	character: u8,
	color_code: ColorCode
}

struct Buffer {
	chars: [[ScreenChar; SCREEN_WIDTH]; SCREEN_HEIGHT]
}

pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: Cell<ColorCode>,
    buffer: Unique<Buffer>,
}

impl Writer {
	pub fn new() -> Writer {
		Writer {
            row_position: 0,
	        column_position: 0,
	        color_code: Cell::new(ColorCode::new(Color::LIGHT_GREEN, Color::BLACK)),
	        buffer: unsafe { Unique::new(0xb8000 as *mut _) },
    	}
	}

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= SCREEN_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                self.buffer().chars[row][col] = ScreenChar {
                    character: byte,
                    color_code: self.color_code.get(),
                };
                self.column_position += 1;
            }
        }
    }

    pub fn clear_screen(&mut self) {
    	for row in 0..(SCREEN_HEIGHT-1) {
    		self.clear_row(row)
    	}
    }


    fn buffer(&mut self) -> &mut Buffer {
        unsafe{ self.buffer.get_mut() }
    }

    fn new_line(&mut self) {
        if self.row_position < SCREEN_HEIGHT - 1 { 
            self.row_position += 1; 
        } else {
    	    for row in 0..(SCREEN_HEIGHT-1) {
    	        let buffer = self.buffer();
    	        buffer.chars[row] = buffer.chars[row + 1]
    	    }
    	    self.clear_row(SCREEN_HEIGHT-1);
	    }

        self.column_position = 0;
	}

	fn clear_row(&mut self, row: usize) {
	    let blank = ScreenChar {
	        character: b' ',
	        color_code: self.color_code.get(),
	    };
	    self.buffer().chars[row] = [blank; SCREEN_WIDTH];
	}
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
	    for byte in s.bytes() {
	      self.write_byte(byte)
	    }

	    Ok(())
	}
}

pub fn clear_screen() {
	WRITER.lock().clear_screen()
}


pub fn switch_color(front: Color, back: Color) {
	WRITER.lock().color_code.set(ColorCode::new(front, back))
} 