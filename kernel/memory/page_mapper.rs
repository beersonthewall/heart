use core::arch::asm;
use core::ptr::NonNull;

use crate::memory::{
    frame_alloc::FrameAllocator, PhysicalAddress, PhysicalAddressRange, VirtualAddress,
    VirtualAddressRange, PAGE_SIZE,
};

const TABLE_SIZE: usize = 512;

struct Table {
    data: [usize; TABLE_SIZE],
    next: Option<NonNull<Table>>,
}

impl Table {
    fn new() -> Self {
        Self {
            data: [0; TABLE_SIZE],
            next: None,
        }
    }
}

pub struct PageMapper {
    p4_table: [usize; TABLE_SIZE],
    p3_table: Table,
    p2_table: Table,
    pt: Table,
}

impl PageMapper {
    pub fn new() -> Self {
        Self {
            p4_table: [0; TABLE_SIZE],
            p3_table: Table::new(),
            p2_table: Table::new(),
            pt: Table::new(),
        }
    }

    pub fn map(
        &mut self,
        virt: VirtualAddressRange,
        phys: PhysicalAddressRange,
        alloc: &mut FrameAllocator,
    ) {
        // TODO actually map the things :)
    }

    pub fn write_cr3(&self) {
        // TODO this may not work since I've started the frame allocator after the kernel.
        // we'll want to make sure that frames allocated from it are identity mapped.
        // or find an alternative way to get the physical address.
        // Since the kernel is identity mapped, this retrieves the physical address.
        /*let pml4 = self.p4_table.as_ptr();
        unsafe {
            asm!("mov cr3, {}", in(reg) pml4.offset(0));
        }*/
    }
}

pub fn flush_tlb(range: VirtualAddressRange) {
    // TODO actually flush the tlb :)
}
