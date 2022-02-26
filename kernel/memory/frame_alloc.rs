use super::PAGE_SIZE;
use x86_64::addr::PhysAddr;
use x86_64::structures::paging::page::Size4KiB;
use x86_64::structures::paging::frame::PhysFrame;
use x86_64::structures::paging::FrameAllocator;

pub struct FA {
    free: PhysFrame<Size4KiB>,
}

impl FA {
    pub fn new(
        start: u64,
    ) -> Self {
        Self {
            free: PhysFrame::from_start_address(PhysAddr::new(start)).unwrap(),
        }
    }

}

unsafe impl FrameAllocator<Size4KiB> for FA {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let f = self.free;
        self.free = self.free + PAGE_SIZE as u64;
        Some(f)
    }
}
