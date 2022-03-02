use super::addr::VirtualAddress;
use super::PAGE_SIZE;

#[derive(Clone, Copy)]
pub struct Page {
    pub page_number: usize,
}

impl Page {
    #[inline]
    pub fn virtual_address(&self) -> VirtualAddress {
        VirtualAddress::new(self.page_number * PAGE_SIZE)
    }

    #[inline]
    pub fn pml4_offset(&self) -> usize {
        (self.virtual_address().0 >> 39) & 0x1FF
    }

    #[inline]
    pub fn pdpt_offset(&self) -> usize {
        (self.virtual_address().0 >> 30) & 0x1FF
    }

    #[inline]
    pub fn pd_offset(&self) -> usize {
        (self.virtual_address().0 >> 21) & 0x1FF
    }

    #[inline]
    pub fn pt_offset(&self) -> usize {
        (self.virtual_address().0 >> 12) & 0x1FF
    }

    pub fn from_virtual_address(vaddr: VirtualAddress) -> Self {
        Self {
            page_number: vaddr.0 / PAGE_SIZE,
        }
    }
}
