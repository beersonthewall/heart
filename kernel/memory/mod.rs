pub mod addr;
pub mod frame;
pub mod heap;
pub mod page;

mod linked_list_heap;

use super::multiboot::MultibootInfo;
use frame::Frame;

#[allow(dead_code)]
#[derive(Debug)]
pub enum PagingError {
    Unknown,
}

/// Frame Allocation trait to enable the page_mapper functions to use either
/// the bootstrap frame allocator or the regular frame allocator.
pub trait FrameAllocatorAPI {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub fn init(multiboot_addr: usize, bootstrap_frame_alloc_start: usize, heap_start_virtual: usize) {
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

    super::arch::memory::init(bootstrap_frame_alloc_start, &multiboot_info, multiboot_addr);
    heap::init(heap_start_virtual);
    log!("memory module init complete.");
}
