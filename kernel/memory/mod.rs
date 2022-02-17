pub mod addr;

mod frame;
mod frame_alloc;
mod page;
mod page_mapper;

use core::mem::size_of;

use crate::multiboot::MultibootInfo;
use addr::{
    PhysicalAddress,
    PhysicalAddressRange,
    VirtualAddress,
    VirtualAddressRange,
};
use frame::Frame;
use frame_alloc::FrameAllocator;
use page::Page;
use page_mapper::PageMapper;

const PAGE_SIZE: usize = 4096;

pub fn init(multiboot_addr: usize) {
    // Identity map kernel and multiboot information struct, but notably *not* the maps
    // referenced by the multiboot info struct. We can't read it yet so we don't know
    // where they are located in physical memory.

    let kernel_virtual_range = VirtualAddressRange::new(VirtualAddress::new(0xFFFFFFFF80000000), 4_000_000);
    let kernel_physical_range = PhysicalAddressRange::new(PhysicalAddress::new(0xFFFFFFFF80000000), 4_000_000);
    let multiboot_virtual_range =
        VirtualAddressRange::new(VirtualAddress::new(multiboot_addr), size_of::<MultibootInfo>());
    let multiboot_physical_range =
        PhysicalAddressRange::new(PhysicalAddress::new(multiboot_addr), size_of::<MultibootInfo>());

    let mut frame_allocator = FrameAllocator::new(
        multiboot_physical_range.base,
        multiboot_physical_range.base + multiboot_physical_range.size,
        kernel_physical_range.base,
        kernel_physical_range.base + kernel_physical_range.size,
        PhysicalAddress::new(1 * 1000 * 1000 * 1000),
    );

    let mut page_mapper = PageMapper::init_kernel_table();

    let test_frame = Frame { frame_number: 0x55_55_BB_00_BB_BB_BB_BB / PAGE_SIZE };
    let test_page = Page { page_number: 0xAA_AA_AA_AA_AA_AA_A0_00 / PAGE_SIZE };
    let addr: usize = 0b1111111111111111_111111110_111111110_111111110_111111110_000000000000;
    let addr: usize = 0b1111111111111111_111111110_111111110_111111110_111111110_000000000000;
    let addr = addr as *const u64;

    page_mapper.map(test_page, test_frame, &mut frame_allocator);
}
