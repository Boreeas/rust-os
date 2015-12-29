use core::ptr::Unique;
use memory::FrameAllocator;
use memory::PAGE_SIZE;
use spin::Mutex;
pub use memory::Frame;

mod entry;
mod table;

use self::entry::*;
use self::table::*;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;


const P4_TABLE: PageTableHead = unsafe { PageTableHead::new() };
pub const P4_TABLE_EXCL: Mutex<PageTableHead> = Mutex::new(P4_TABLE);








#[derive(Debug)]
struct Page {
    number: usize
}

impl Page {
    pub fn for_address(addr: VirtualAddress) -> Page {
        assert!(addr < 0x0000_8000_0000_0000    // Bit 48 unset and no sign ext
             || addr >= 0xffff_8000_0000_0000,  // Bit 48 set and sign ext
             "Invalid address 0x{:x}", addr);

        Page { number: addr / PAGE_SIZE }
    }

    fn for_table_indices(p4: usize, p3: usize, p2: usize, p1: usize) -> Page {
        assert!(p4 < ENTRY_COUNT 
            && p3 < ENTRY_COUNT 
            && p2 < ENTRY_COUNT 
            && p1 < ENTRY_COUNT);

        Page {
            number: (p4 << 27) | (p3 << 18) | (p2 << 9) | p1 
        }
    }

    pub fn first_addr(&self) -> VirtualAddress {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        self.number & 0o777
    }
}








pub struct PageTableHead {
    p4: Unique<PageTable<Level4>>
}

impl PageTableHead {
    const unsafe fn new() -> PageTableHead {
        PageTableHead {
            p4: Unique::new(P4_TABLE_MASK as *mut _)
        }
    }

    fn get_p4(&self) -> &PageTable<Level4> {
        unsafe { self.p4.get() }
    }

    fn get_p4_mut(&mut self) -> &mut PageTable<Level4> {
        unsafe { self.p4.get_mut() }
    }


    pub fn translate(&self, addr: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = addr % PAGE_SIZE;
        self.translate_page(Page::for_address(addr))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        let p4 = self.get_p4();
        let p3 = p4.next_table(page.p4_index());
        
        let huge_page = || { 
            return p3.and_then(|p3| {

            let p3_entry = &p3[page.p3_index()];
            if let Some(frame) = p3_entry.target_frame() {
                if p3_entry.flags().contains(HUGE_PAGE) {
                    // Page must be 1GiB aligned
                    assert!(frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0,
                        "Frame number 0x{:x} (addr 0x{:x}) not valid for 1GiB page",
                        frame.number, frame.first_addr());
                    return Some(Frame {
                        number: frame.number 
                                + page.p2_index() * ENTRY_COUNT 
                                + page.p1_index()
                    })
                }
            }

            if let Some(p2) = p3.next_table(page.p3_index()) {
                let p2_entry = &p2[page.p2_index()];
                if let Some(frame) = p2_entry.target_frame() {
                    if p2_entry.flags().contains(HUGE_PAGE) {
                        // Page must be 2MiB aligned
                        assert!(frame.number % ENTRY_COUNT == 0,
                            "Frame number 0x{:x} (addr 0x{:x}) not valid for 2MiB page", 
                            frame.number, frame.first_addr());
                        return Some(Frame {
                            number: frame.number + page.p1_index()
                        })
                    }
                }
            }
            None
            });
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
          .and_then(|p2| p2.next_table(page.p2_index()))
          .and_then(|p1| p1[page.p1_index()].target_frame())
          .or_else(huge_page)
    }


    fn map_to<A>(&mut self, 
        page: &Page, 
        frame: Frame, 
        flags: EntryFlags, 
        allocator: &mut A)
        where A: FrameAllocator {

        let p4  = self.get_p4_mut();
        let mut p3 = p4.next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT | WRITEABLE);
    }

    fn map<A>(&mut self, page: &Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator {

        let frame = allocator.alloc().expect("Out of frames");
        self.map_to(page, frame, flags, allocator);
    }

    fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, alloc: &mut A)
        where A: FrameAllocator {

        let page = Page::for_address(frame.first_addr());
        self.map_to(&page, frame, flags, alloc);
    }

    fn unmap<A>(&mut self, page: Page, alloc: &mut A) where A: FrameAllocator {
        assert!(self.translate(page.first_addr()).is_some());

        let p1 = self.get_p4_mut()
                     .next_table_mut(page.p4_index())
                     .and_then(|p3| p3.next_table_mut(page.p3_index()))
                     .and_then(|p2| p2.next_table_mut(page.p2_index()))
                     .expect("mapping code does not support huge pages");
        let frame = p1[page.p1_index()].target_frame().unwrap();
        p1[page.p1_index()].set_unused();
        unsafe {
            ::x86::tlb::flush(page.first_addr());
        }
        // TODO free p(1,2,3) table if empty
        alloc.dealloc(frame);
    }
}

pub fn simple_id_map<A: FrameAllocator>(frame: Frame, alloc: &mut A) {
    P4_TABLE.identity_map(frame, EntryFlags::empty(), alloc)
}

pub fn alloc_any<A>(alloc: &mut A) -> &'static mut [u8; PAGE_SIZE] where A: FrameAllocator {
    let mutex = P4_TABLE_EXCL;
    let table_head = mutex.lock();
    let p4 = table_head.get_p4();

    for p4_idx in 0..ENTRY_COUNT {
        if let Some(p3) = p4.next_table(p4_idx) { 
            for p3_idx in 0..ENTRY_COUNT {
                if let Some(p2) = p3.next_table(p3_idx) {
                    for p2_idx in 0..ENTRY_COUNT {
                        if let Some(p1) = p2.next_table(p2_idx) {
                            for p1_idx in 0..ENTRY_COUNT {
                                if p1[p1_idx].is_unused() {
                                    let page = Page::for_table_indices(
                                        p4_idx, 
                                        p3_idx, 
                                        p2_idx, 
                                        p1_idx
                                    );
                                    P4_TABLE.map(&page, EntryFlags::empty(), alloc);
                                    return unsafe {
                                        &mut *((page.number * PAGE_SIZE) as *mut _) 
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } 
    }

    panic!("Out of memory");
}