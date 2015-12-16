mod port;

pub use self::port::{Port, UnsafePort};

pub struct Pic {
	offset: u8,
	command: UnsafePort<u8>,
	data: UnsafePort<u8>
}


impl Pic {
	pub fn handles_interrupt() {}
}