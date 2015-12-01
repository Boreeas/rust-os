use core::marker::PhantomData;

pub trait PicValue {
	unsafe fn write(port: u16, value: Self);
	unsafe fn read(port: u16) -> Self;
}

impl PicValue for u8 {
	unsafe fn write(port: u16, value: u8) {
		asm!("outb %al, %dx" 
			:
			: "{dx}"(port), "{al}"(value) 
			:
			: "volatile");
	}

	unsafe fn read(port: u16) -> u8 {
		let res;

		asm!("inb %dx, %al" 
			: "={al}"(res)
			: "{dx}"(port) 
			: "{al}"
			: "volatile");

		res
	}
}

impl PicValue for u16 {
	unsafe fn write(port: u16, value: u16) {
		asm!("outw %ax, %dx"
			:
			: "{dx}"(port), "{ax}"(value)
			:
			: "volatile");
	}

	unsafe fn read(port: u16) -> u16 {
		let res;

		asm!("inw %dx, %ax"
			: "={ax}"(res)
			: "{dx}"(port)
			: "{ax}"
			: "volatile");

		res
	}
}

impl PicValue for u32 {
	unsafe fn write(port: u16, value: u32) {
		asm!("outl %eax, %dx"
			: 
			: "{dx}"(port), "{eax}"(value)
			:
			: "volatile")
	}

	unsafe fn read(port: u16) -> u32 {
		let res;

		asm!("inl %dx, %eax"
			: "={eax}"(res)
			: "{dx}"(port)
			: "{eax}"
			: "volatile");

		res
	}
}

pub struct Port<T: PicValue> {
	port: u16,
	_phantomdata: PhantomData<T>
}

impl <T: PicValue> Port<T> {
	pub const unsafe fn new(port: u16) -> Port<T> {
		Port {
			port: port,
			_phantomdata: PhantomData
		}
	}

	pub fn read(&self) -> T {
		unsafe { 
			T::read(self.port)
		}
	}

	pub fn write(&self, value: T) {
		unsafe {
			T::write(self.port, value)
		}
	}
}