use memory::Frame;

// Page aligned address of the next frame
// Bits 12-51 of a page entry
const ADDR_MASK: usize = 0x000f_ffff_ffff_f000;


pub struct Entry(u64);

bitflags! {
    flags EntryFlags: u64 {
        const PRESENT =         1 <<  0,
        const WRITEABLE =       1 <<  1,
        const USER_ACCESSIBLE = 1 <<  2,
        const WRITE_THROUGH =   1 <<  3,
        const NO_CACHE =        1 <<  4,
        const ACCESSED =        1 <<  5,
        const DIRTY =           1 <<  6,
        const HUGE_PAGE =       1 <<  7,
        const GLOBAL =          1 <<  8,
        const NO_EXECUTE =      1 << 63
    }
}

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn target_frame(&self) -> Option<Frame> {
        if self.flags().contains(PRESENT) {
            Some(Frame::for_address(self.0 as usize & ADDR_MASK))
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        debug_assert!(frame.first_addr() & !ADDR_MASK == 0);
        self.0 = (frame.first_addr() as u64) | flags.bits();
    }
}
