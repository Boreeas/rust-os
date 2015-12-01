use spin::Mutex;
use pic::Port;

pub static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new());
static LOOPUP_TABLE: [Input; 0] = [
	
];
static EXTENDED_LOOKUP_TABLE_1: [Input; 0] = [

];

pub enum Input {
	Pressed(Key),
	Released(Key),
}

#[derive(Debug)]
pub enum Key {
	Char(char),
	Meta(MetaKey)
}

#[derive(Debug)]
pub enum MetaKey {
	F0,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	F10,
	F11,
	F12,
	Tab,
	LeftAlt,
	LeftShift,
	LeftCtrl,
	Home,
	Esc,
	Backspace,
	Enter,
	RightShift,
	Print,
	Alt,
	CapsLock
}

pub struct Keyboard {
	port: Port<u8>
}

impl Keyboard {
	pub const fn new() -> Keyboard {
		Keyboard { 
			port: unsafe { Port::new(0x60) }
		}
	}

	pub fn poll(&self) -> Option<Input> {
		use self::Input::*;
		use self::Key::*;
		use self::MetaKey::*;

		match self.port.read() {
			0x01 => Some(Pressed(Meta(Esc))),
			0x02 => Some(Pressed(Char('1'))),
			0x03 => Some(Pressed(Char('2'))),
			0x04 => Some(Pressed(Char('3'))),
			0x05 => Some(Pressed(Char('4'))),
			0x06 => Some(Pressed(Char('5'))),
			0x07 => Some(Pressed(Char('6'))),
			0x08 => Some(Pressed(Char('8'))),
			0x09 => Some(Pressed(Char('8'))),
			0x0a => Some(Pressed(Char('9'))),
			0x0b => Some(Pressed(Char('0'))),
			0x0c => Some(Pressed(Char('ß'))),
			0x0d => Some(Pressed(Char('´'))),
			0x0e => Some(Pressed(Meta(Backspace))),
			0x0f => Some(Pressed(Meta(Tab))),
			0x10 => Some(Pressed(Char('q'))),
			0x11 => Some(Pressed(Char('w'))),
			0x12 => Some(Pressed(Char('e'))),
			0x13 => Some(Pressed(Char('r'))),
			0x14 => Some(Pressed(Char('t'))),
			0x15 => Some(Pressed(Char('z'))),
			0x16 => Some(Pressed(Char('u'))),
			0x17 => Some(Pressed(Char('i'))),
			0x18 => Some(Pressed(Char('o'))),
			0x19 => Some(Pressed(Char('p'))),
			0x1a => Some(Pressed(Char('ü'))),
			0x1b => Some(Pressed(Char('+'))),
			0x1c => Some(Pressed(Meta(Enter))),
			0x1d => Some(Pressed(Meta(LeftCtrl))),
			0x1e => Some(Pressed(Char('a'))),
			0x1f => Some(Pressed(Char('s'))),
			0x20 => Some(Pressed(Char('d'))),
			0x21 => Some(Pressed(Char('f'))),
			0x22 => Some(Pressed(Char('g'))),
			0x23 => Some(Pressed(Char('h'))),
			0x24 => Some(Pressed(Char('j'))),
			0x25 => Some(Pressed(Char('k'))),
			0x26 => Some(Pressed(Char('l'))),
			0x27 => Some(Pressed(Char('ö'))),
			0x28 => Some(Pressed(Char('ä'))),
			0x29 => Some(Pressed(Char('^'))),
			0x2a => Some(Pressed(Meta(LeftShift))),
			0x2b => Some(Pressed(Char('#'))),
			0x2c => Some(Pressed(Char('y'))),
			0x2d => Some(Pressed(Char('x'))),
			0x2e => Some(Pressed(Char('c'))),
			0x2f => Some(Pressed(Char('v'))),
			0x30 => Some(Pressed(Char('b'))),
			0x31 => Some(Pressed(Char('n'))),
			0x32 => Some(Pressed(Char('m'))),
			0x33 => Some(Pressed(Char(','))),
			0x34 => Some(Pressed(Char('.'))),
			0x35 => Some(Pressed(Char('-'))),
			0x36 => Some(Pressed(Meta(RightShift))),
			0x37 => Some(Pressed(Meta(Print))),
			0x38 => Some(Pressed(Meta(Alt))),
			0x39 => Some(Pressed(Char(' '))),
			0x3a => Some(Pressed(Meta(CapsLock))),
			0x3b => Some(Pressed(Meta(F1))),
			0x3c => Some(Pressed(Meta(F2))),
			0x3d => Some(Pressed(Meta(F3))),
			0x3e => Some(Pressed(Meta(F4))),
			0x3f => Some(Pressed(Meta(F5))),
			_ 	 => None
		}
	}

	pub fn next_key(&self) -> Key {
		use self::Input::*;

		loop {
			match self.poll() {
				Some(Pressed(k))	=> return k,
				_ 					=> {}
			}
		}
	}
}