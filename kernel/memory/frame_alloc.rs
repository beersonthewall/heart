use crate::memory::{PhysicalAddress, Frame};

pub struct FrameAllocator {
    free: Frame,
}

impl FrameAllocator {
    pub fn new(
        start: PhysicalAddress,
    ) -> Self {
        Self {
            free: Frame::from_physical_address(&start),
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let f = self.free;
        self.free = Frame { frame_number: f.frame_number + 1 };
        Some(f)
    }
}
