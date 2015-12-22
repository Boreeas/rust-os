pub use self::area_frame_allocator::AreaFrameAllocator;
mod area_frame_allocator;
mod paging;

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

	pub fn first_addr(&self) -> usize {
		self.number * PAGE_SIZE
	}

	pub fn last_addr(&self) -> usize {
		(self.number + 1) * PAGE_SIZE - 1
	}

	pub fn contains_addr(&self, addr: usize) -> bool {
		addr / PAGE_SIZE == self.number
	}

	pub fn next(&self) -> Frame {
		Frame { number: self.number + 1 }
	}
}

pub trait FrameAllocator {
	fn alloc(&mut self) -> Option<Frame>;
	fn dealloc(&mut self, frame: Frame);
}

#[test]
fn test_frame() {
	let f = Frame::new(0);

	assert_eq!(f.number, 0);
	assert_eq!(f.first_addr(), 0);
	assert_eq!(f.last_addr(), PAGE_SIZE - 1);
	assert_eq!(f.next().first_addr(), f.last_addr() + 1);
	assert!(f.contains_addr(PAGE_SIZE / 2));
}

#[test]
fn test_frame_for_addr() {
	let f = Frame::for_address(3 * PAGE_SIZE + 3);

	assert_eq!(f.number, 3);
	assert_eq!(f.first_addr(), 3 * PAGE_SIZE);
	assert_eq!(f.last_addr(), 4 * PAGE_SIZE - 1);
	assert!(f.contains_addr(f.first_addr()));
	assert!(f.contains_addr(f.last_addr()));
}