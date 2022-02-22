pub mod addr;

mod frame;
mod frame_alloc;
mod page;
mod page_mapper;
mod page_table;

use addr::{PhysicalAddress};
use frame::Frame;
use frame_alloc::FrameAllocator;
use page::Page;
use page_mapper::PageMapper;

const PAGE_SIZE: usize = 4096;

pub fn init(_multiboot_addr: usize) {

    let mut frame_allocator = FrameAllocator::new(
        PhysicalAddress::new(1 * 1000 * 1000 * 1000),
    );

    let mut page_mapper = PageMapper::init_kernel_table();

    let test_frame = Frame { frame_number: 0x55_55_BB_00_BB_BB_BB_BB / PAGE_SIZE };
    let test_page = Page { page_number: 0xAA_AA_AA_AA_AA_AA_A0_00 / PAGE_SIZE };

    page_mapper.map(test_page, test_frame, &mut frame_allocator);
}
