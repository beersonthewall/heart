mod frame_alloc;

use frame_alloc::FA;

use x86_64::addr::{PhysAddr, VirtAddr};
use x86_64::structures::paging::frame::PhysFrame;
use x86_64::structures::paging::mapper::Mapper;
use x86_64::structures::paging::mapper::RecursivePageTable;
use x86_64::structures::paging::page::Page;
use x86_64::structures::paging::page::Size4KiB;
use x86_64::structures::paging::page_table::{PageTable, PageTableFlags};

const PAGE_SIZE: usize = 4096;

pub fn init(multiboot_addr: usize) {
    let level4_table_addr: u64 = 0xffff_ff7f_bfdf_e000;
    let level4: &mut PageTable = unsafe { &mut *(level4_table_addr as *mut PageTable) };
    let mut recursive_table =
        RecursivePageTable::new(level4).expect("failed to create recursive table");
    let page = Page::<Size4KiB>::from_start_address(VirtAddr::new(1 * 1024 * 1024 * 1024)).unwrap();
    let frame = PhysFrame::from_start_address(PhysAddr::new(8 * 1024 * 1024)).unwrap();
    let mut frame_allocator = FA::new(4 * 1024 * 1024);
    unsafe {
        recursive_table
            .map_to_with_table_flags(
                page,
                frame,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE
                    | PageTableFlags::NO_EXECUTE
                    | PageTableFlags::NO_CACHE,
                PageTableFlags::USER_ACCESSIBLE,
                &mut frame_allocator,
            )
            .unwrap()
            .flush();
    }
    log!("Success!!!!");
}
