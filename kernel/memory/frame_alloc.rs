use crate::memory::{Frame, PhysicalAddress};

pub struct FrameAllocator {
    free: Frame,
}

impl FrameAllocator {
    pub fn new(start: PhysicalAddress) -> Self {
        log!("start: {start}");
        Self {
            free: Frame::from_physical_address(&start),
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let f = self.free;
        self.free = Frame {
            frame_number: f.frame_number + 1,
        };
        let addr = f.physical_address().0;
        log!("allocating frame: {addr:x}");
        Some(f)
    }
}
