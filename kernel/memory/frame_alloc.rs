use crate::memory::{PhysicalAddress, PAGE_SIZE};

#[derive(Copy, Clone)]
struct Frame(PhysicalAddress);

pub struct FrameAllocator {
    multiboot_start: Frame,
    multiboot_end: Frame,
    kernel_start: Frame,
    kernel_end: Frame,
    free: Frame,
}

impl FrameAllocator {
    pub fn new(
        multiboot_start: PhysicalAddress,
        multiboot_end: PhysicalAddress,
        kernel_start: PhysicalAddress,
        kernel_end: PhysicalAddress,
        start: PhysicalAddress,
    ) -> Self {
        Self {
            multiboot_start: Frame(multiboot_start),
            multiboot_end: Frame(multiboot_end),
            kernel_start: Frame(kernel_start),
            kernel_end: Frame(kernel_end),
            free: Frame(start),
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let f = self.free;
        self.free = Frame(f.0 + 1);

        assert!(!(self.kernel_start.0..self.kernel_end.0 + 1).contains(&self.free.0));
        assert!(!(self.multiboot_start.0..self.multiboot_end.0 + 1).contains(&self.free.0));

        Some(f)
    }

    pub fn deallocate_frame(&mut self, f: Frame) {
        // Do nothing, this is a bump allocator. :)
        // TODO write new allocator that can actually free frames.
        // TODO create FrameAllocator trait
    }
}
