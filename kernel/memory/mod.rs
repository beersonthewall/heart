pub mod addr;

mod frame;
mod frame_alloc;
mod page;
mod page_mapper;
mod page_table;

use addr::{PhysicalAddress, VirtualAddress};
use frame::Frame;
use frame_alloc::FrameAllocator;
use page::Page;
use page_mapper::PageMapper;

const PAGE_SIZE: usize = 4096;

enum PagingError {
    Unknown,
}

pub fn init(_multiboot_addr: usize) {
    let mut frame_allocator = FrameAllocator::new(PhysicalAddress::new(10_000 * PAGE_SIZE));

    let mut page_mapper = PageMapper::init_kernel_table();

    let test_frame = Frame {
        frame_number: 0x4e20000 / PAGE_SIZE,
    };
    let test_page = Page::from_virtual_address(VirtualAddress::new(0x0000_1FFF_0000_0000));

    page_mapper.map(test_page, test_frame, &mut frame_allocator);
    log!("Success!");

    let test_value: &mut u64 = unsafe { &mut *(test_page.virtual_address().0 as *mut u64) };
    log!("test_value before: {test_value}");
    *test_value = 100;
    log!("test_value after: {test_value}");
}
