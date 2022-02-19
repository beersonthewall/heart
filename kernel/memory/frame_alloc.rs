use crate::memory::{PhysicalAddress, PAGE_SIZE, Frame};

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
            multiboot_start: Frame::from_physical_address(&multiboot_start),
            multiboot_end: Frame::from_physical_address(&multiboot_end),
            kernel_start: Frame::from_physical_address(&kernel_start),
            kernel_end: Frame::from_physical_address(&kernel_end),
            free: Frame::from_physical_address(&start),
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let f = self.free;
        self.free = Frame { frame_number: f.frame_number + 1 };
        Some(f)
    }
}
