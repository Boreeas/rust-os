use memory::{FrameAllocator, Frame};
use multiboot2::{MemoryArea, MemoryAreaIter};

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
    apic_frame: Frame,
}

trait MemoryAreaExt {
    fn get_first_frame(&self) -> Frame;
    fn get_last_frame(&self) -> Frame;
}

impl MemoryAreaExt for MemoryArea {
    fn get_first_frame(&self) -> Frame {
        Frame::for_address(self.base_addr as usize)
    }

    fn get_last_frame(&self) -> Frame {
        Frame::for_address((self.base_addr + self.length - 1) as usize)
    }
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: usize,
               kernel_end: usize,
               multiboot_start: usize,
               multiboot_end: usize,
               apic_loc: usize,
               memory_areas: MemoryAreaIter)
               -> AreaFrameAllocator {

        let mut alloc = AreaFrameAllocator {
            next_free_frame: Frame::for_address(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::for_address(kernel_start),
            kernel_end: Frame::for_address(kernel_end),
            multiboot_start: Frame::for_address(multiboot_start),
            multiboot_end: Frame::for_address(multiboot_end),
            apic_frame: Frame::for_address(apic_loc),
        };

        alloc.choose_next_area();
        alloc
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas
                                .clone()
                                .filter(|area| area.get_last_frame() > self.next_free_frame)
                                .min_by(|obj, reference| obj.base_addr.cmp(&reference.base_addr));

        if let Some(area) = self.current_area {
            // We found a new area
            if self.next_free_frame < area.get_first_frame() {
                self.next_free_frame = area.get_first_frame()
            }
        }
    }
}

impl FrameAllocator for AreaFrameAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        if self.current_area.is_none() {
            return None;
        }

        let frame = self.next_free_frame;
        let last_for_current_area = self.current_area.unwrap().get_last_frame();

        if self.next_free_frame >= last_for_current_area {
            self.choose_next_area();
        } else {
            self.next_free_frame = self.next_free_frame.next();
        }

        loop {
            if self.next_free_frame >= self.kernel_start &&
               self.next_free_frame <= self.kernel_end {
                self.next_free_frame = self.kernel_end.next();
            } else if self.next_free_frame >= self.multiboot_start &&
               self.next_free_frame <= self.multiboot_end {
                self.next_free_frame = self.multiboot_end.next();
            } else if self.next_free_frame == self.apic_frame {
                self.next_free_frame = self.apic_frame.next()
            } else {
                break;
            }
        }

        return Some(frame);
    }

    fn dealloc(&mut self, _: Frame) {
        // unimplemented!()
    }
}
