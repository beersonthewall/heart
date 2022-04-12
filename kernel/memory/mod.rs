pub mod addr;

mod frame;
mod frame_alloc;
mod page;
mod page_mapper;
mod page_table;

use super::multiboot::MultibootInfo;
use addr::{PhysicalAddress, VirtualAddress};
use frame::Frame;
use frame_alloc::{BootstrapFrameAllocator, FrameAllocator};
use page::Page;
use page_mapper::PageMapper;

pub const PAGE_SIZE: usize = 4096;

#[allow(dead_code)]
#[derive(Debug)]
pub enum PagingError {
    Unknown,
}

fn test_page_mapper(
    page_mapper: &mut PageMapper,
    frame_allocator: &mut BootstrapFrameAllocator,
    multiboot_addr: usize,
) {
    let test_frame = Frame {
        frame_number: 0x4e20000 / PAGE_SIZE,
    };
    let test_page = Page::from_virtual_address(VirtualAddress::new(0x0000_1FFF_0000_0000));

    page_mapper
        .map(test_page, test_frame, frame_allocator)
        .unwrap();
    log!("Success!");

    let test_value: &mut u64 = unsafe { &mut *(test_page.virtual_address().0 as *mut u64) };
    log!("test_value before: {test_value}");
    *test_value = 100;
    log!("test_value after: {test_value}");

    log!("test is_mapped");
    assert!(page_mapper.is_mapped(test_page));

    log!("unmapping page");
    page_mapper
        .unmap(test_page, test_frame, frame_allocator)
        .unwrap();
    log!("page mapper test complete :)");

    log!("loading multiboot information and initializing kernel memory map");
    let mboot_vaddr = VirtualAddress(multiboot_addr);
    let mboot_page = Page::from_virtual_address(mboot_vaddr);
    assert!(
        page_mapper.is_mapped(mboot_page),
        "Multiboot struct should be identity mapped."
    );
}

pub fn init(multiboot_addr: usize, heap_start: usize) {
    let multiboot_info = MultibootInfo::new(multiboot_addr);
    log!("flags: 0x{:x}", multiboot_info.flags());
    log!("mem_lower: 0x{:x}", multiboot_info.mem_lower());
    log!("mem_upper: 0x{:x}", multiboot_info.mem_upper());
    log!("mmap_addr: 0x{:x}", multiboot_info.mmap_addr());
    log!("mmap_length: 0x{:x}", multiboot_info.mmap_length());

    let mmap_iter = multiboot_info.mmap_iter();
    for entry in mmap_iter {
        log!(
            "size: {}, base_addr: 0x{:x}, length: 0x{:x}, entry_type: {:?}",
            entry.size(),
            entry.base_addr(),
            entry.length(),
            entry.entry_type()
        );
    }

    let mut bootstrap_frame_allocator =
        BootstrapFrameAllocator::new(PhysicalAddress::new(heap_start));
    let mut page_mapper = PageMapper::init_kernel_table();

    test_page_mapper(
        &mut page_mapper,
        &mut bootstrap_frame_allocator,
        multiboot_addr,
    );

    let _frame_alloc =
        FrameAllocator::new(bootstrap_frame_allocator, &multiboot_info, &mut page_mapper);
}
