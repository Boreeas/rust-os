pub use self::area_frame_allocator::AreaFrameAllocator;
mod area_frame_allocator;

pub const PAGE_SIZE: usize = 4096;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
	number: usize
}

impl Frame {
	pub fn new(number: usize) -> Frame {
		Frame { number: number }
	}

	pub fn for_address(addr: usize) -> Frame {
		Frame { number: addr / PAGE_SIZE }
	}

	pub fn next(&self) -> Frame {
		Frame { number: self.number + 1 }
	}
}

pub trait FrameAllocator {
	fn alloc(&mut self) -> Option<Frame>;
	fn dealloc(&mut self, frame: Frame);
}