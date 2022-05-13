pub mod frame_allocator;
pub mod page_mapper;
pub mod page_table;

use crate::memory::addr::{PhysicalAddress, VirtualAddress};
use crate::memory::frame::Frame;
use crate::memory::page::Page;
use crate::memory::FrameAllocatorAPI;
use crate::memory::PagingError;
use crate::multiboot::MultibootInfo;
use frame_allocator::{BootstrapFrameAllocator, FrameAllocator, FrameAllocatorInner};
use page_mapper::{KernelPageMapper, PageMapper};
use spin::mutex::Mutex;

pub const PAGE_SIZE: usize = 4096;

static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::new();
static mut KERNEL_PAGE_TABLE: KernelPageMapper = KernelPageMapper::new();

pub fn init(heap_start: usize, multiboot_info: &MultibootInfo, multiboot_addr: usize) {
    let mut bootstrap_frame_allocator =
        BootstrapFrameAllocator::new(PhysicalAddress::new(heap_start));
    let mut page_mapper = PageMapper::init_kernel_table();

    test_page_mapper(
        &mut page_mapper,
        &mut bootstrap_frame_allocator,
        multiboot_addr,
    );

    let fa =
        FrameAllocatorInner::new(bootstrap_frame_allocator, multiboot_info, &mut page_mapper);
    unsafe {
        FRAME_ALLOCATOR.inner = Mutex::new(Some(fa));
        KERNEL_PAGE_TABLE.inner = Mutex::new(Some(page_mapper));
    }
}

pub fn map(start: VirtualAddress, length: usize) -> Result<(), PagingError> {
    assert!(length % PAGE_SIZE == 0);
    let num_frames = length / PAGE_SIZE;

    for _ in 0..num_frames {
        let page = Page::from_virtual_address(start);
        unsafe {
            let frame = FRAME_ALLOCATOR.allocate_frame().unwrap();
            KERNEL_PAGE_TABLE.map(page, frame, &mut FRAME_ALLOCATOR)?;
        }
    }
    Ok(())
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
