use core::ops::{Index, IndexMut};
use super::frame::Frame;
use super::addr::VirtualAddress;

pub const PTE_PRESENT: u64 = 0b1;
pub const PTE_WRITE: u64 = 0b10;

#[derive(Clone)]
#[repr(C)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn is_used(&self) -> bool {
        self.0 != 0
    }

    pub fn set_frame(&mut self, frame: Frame, options: u64) {
        let value = frame.physical_address() | options;
        log!("set_frame: {value:x}");
        self.0 |= frame.physical_address() | options;
    }
}

const TABLE_SIZE: usize = 512;

#[repr(align(4096))]
#[repr(C)]
pub struct Table {
    entries: [PageTableEntry; TABLE_SIZE],
}

impl Table {
    pub fn from_virtual_address<'a>(address: VirtualAddress) -> &'a mut Table {
        return unsafe { &mut *(address.0 as *mut Table) };
    }
}

impl Index<usize> for Table {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for Table {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}
