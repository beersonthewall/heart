use super::PAGE_SIZE;
use super::addr::PhysicalAddress;

#[derive(Clone, Copy)]
pub struct Frame {
    pub frame_number: usize,
}

impl Frame {
    pub fn from_physical_address(addr: &PhysicalAddress) -> Self {
        Self {
            frame_number: (*addr).0 / PAGE_SIZE,
        }
    }

    pub fn physical_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.frame_number * PAGE_SIZE)
    }
}
