use core::ops::{Index, IndexMut};
use core::marker::PhantomData;
use memory::paging::entry::*;
use memory::FrameAllocator;

pub const ENTRY_COUNT: usize = 512; // Entries in a page table

pub const P4_TABLE_MASK: usize = 0o177777_777_777_777_777_0000;
pub const P3_TABLE_MASK: usize = 0o177777_777_777_777_000_0000;
// ^ index into p4
pub const P2_TABLE_MASK: usize = 0o177777_777_777_000_000_0000;
//                                                ^   ^ index into p3
//                                                \ index into p4
pub const P1_TABLE_MASK: usize = 0o177777_777_000_000_000_0000;
//                                            ^   ^   ^ index into p2
//                                            |   \ index into p3
//                                            \ index into p4


pub trait TableLevel {}
trait HierarchicalLevel: TableLevel {
    type Next: TableLevel;
}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

impl HierarchicalLevel for Level4 {
    type Next = Level3;
}
impl HierarchicalLevel for Level3 {
    type Next = Level2;
}
impl HierarchicalLevel for Level2 {
    type Next = Level1;
}


pub struct PageTable<Lvl: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    _marker: PhantomData<Lvl>,
}

impl<Lvl> Index<usize> for PageTable<Lvl> where Lvl: TableLevel
{
    type Output = Entry;

    fn index(&self, idx: usize) -> &Entry {
        &self.entries[idx]
    }
}

impl<Lvl> IndexMut<usize> for PageTable<Lvl> where Lvl: TableLevel
{
    fn index_mut(&mut self, idx: usize) -> &mut Entry {
        &mut self.entries[idx]
    }
}

impl<Lvl> PageTable<Lvl> where Lvl: TableLevel
{
    fn zero(&mut self) {
        for i in 0..ENTRY_COUNT {
            self.entries[i].set_unused();
        }
    }
}

impl<Lvl> PageTable<Lvl> where Lvl: HierarchicalLevel
{
    fn next_table_address(&self, idx: usize) -> Option<usize> {
        let flags = self.entries[idx].flags();
        if flags.contains(PRESENT) && !flags.contains(HUGE_PAGE) {
            let table = self as *const _ as usize;
            Some(table << 9 | idx << 12)
        } else {
            None
        }
    }

    pub fn next_table(&self, idx: usize) -> Option<&PageTable<Lvl::Next>> {
        self.next_table_address(idx).map(|addr| unsafe { &*(addr as *const _) })
    }

    pub fn next_table_mut(&self, idx: usize) -> Option<&mut PageTable<Lvl::Next>> {
        self.next_table_address(idx).map(|addr| unsafe { &mut *(addr as *mut _) })
    }

    pub fn next_table_create<A>(&mut self, idx: usize, alloc: &mut A) -> &mut PageTable<Lvl::Next>
        where A: FrameAllocator
    {

        if self.next_table(idx).is_none() {
            assert!(!self.entries[idx].flags().contains(HUGE_PAGE));
            let frame = alloc.alloc().expect("Out of frames");
            self.entries[idx].set(frame, PRESENT | WRITEABLE);
            self.next_table_mut(idx).unwrap().zero();
        }

        self.next_table_mut(idx).unwrap()
    }
}
