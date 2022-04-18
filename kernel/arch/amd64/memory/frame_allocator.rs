/*
 * This module contains frame allocators for managing the physical memory of the system.
 */

use super::page_mapper::PageMapper;
use super::PAGE_SIZE;
use crate::memory::addr::PhysicalAddress;
use crate::memory::frame::Frame;
use crate::multiboot::{MMapEntryType, MultibootInfo};

/// Frame Allocation trait to enable the page_mapper functions to use either
/// the bootstrap frame allocator or the regular frame allocator.
pub trait FrameAlloc {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub struct BootstrapFrameAllocator {
    start: PhysicalAddress,
    free: Frame,
}

impl BootstrapFrameAllocator {
    pub fn new(start: PhysicalAddress) -> Self {
        Self {
            start: start,
            free: Frame::from_physical_address(start),
        }
    }

    fn start(self) -> PhysicalAddress {
        self.start
    }

    fn free(self) -> PhysicalAddress {
        self.free.physical_address()
    }
}

impl FrameAlloc for BootstrapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        let f = self.free;
        self.free = Frame {
            frame_number: f.frame_number + 1,
        };
        Some(f)
    }

    // For the bootstrap allocator we're not worried about de-allocating frames.
    fn deallocate_frame(&mut self, _frame: Frame) {}
}

pub struct FrameAllocator<'a> {
    bitmap: &'a mut [u8],
    free_frame_offset: usize,
    free_frame_byte_offset: u8,
}

impl<'a> FrameAllocator<'a> {
    pub fn new(
        mut bootstrap_frame_alloc: BootstrapFrameAllocator,
        info: &MultibootInfo,
        _page_mapper: &mut PageMapper,
    ) -> Self {
        let mut total_bytes_of_memory: usize = 0;
        for entry in info.mmap_iter() {
            if (entry.length() + entry.base_addr()) as usize > total_bytes_of_memory {
                total_bytes_of_memory = (entry.length() + entry.base_addr()) as usize;
            }
        }

        log!("total bytes: 0x{:x}", total_bytes_of_memory);

        let bitmap_sz = (total_bytes_of_memory / PAGE_SIZE) / 8;
        let frames = bitmap_sz / PAGE_SIZE;
        let bitmap_start_frame = bootstrap_frame_alloc.allocate_frame().unwrap();
        log!(
            "Creating frame alloc bitmap, allocating {} frames.",
            frames + 1
        );

        for _ in 1..frames {
            bootstrap_frame_alloc.allocate_frame().unwrap();
        }

        let ptr = bitmap_start_frame.physical_address().0 as *mut u8;
        // FIXME need to map ptr, might be id mapped right now though? just because kernel < 2MB and
        // we mapped 2 MB in start.S (at least for amd64 arch).
        let bitmap = unsafe { core::slice::from_raw_parts_mut(ptr, bitmap_sz) };

        // FIXME: detect & mark regions not in the memory map as reserved.
        // We won't necessarily have all existing memory in the map.
        // FIXME mark frames allocated by BootstrapAllocator as used.
        for entry in info.mmap_iter() {
            if let MMapEntryType::Available = entry.entry_type() {
                continue;
            }

            let base_addr: usize = entry.base_addr() as usize;
            let end_addr: usize = entry.length() as usize + base_addr;

            // We only have granularity to track PAGE_SIZE chunks, so any unavailable
            // ranges that aren't page-aligned will have some extra space marked as
            // unavailable.
            let base_addr = base_addr - (base_addr % PAGE_SIZE);
            let end_addr = end_addr - (end_addr % PAGE_SIZE);

            log!("base_addr: 0x{:x}, end_addr: 0x{:x}", base_addr, end_addr);

            assert!(base_addr % PAGE_SIZE == 0);
            assert!(end_addr % PAGE_SIZE == 0);
            for addr in (base_addr..end_addr).step_by(PAGE_SIZE) {
                let (bitmap_offset, byte_offset) = FrameAllocator::offsets(addr);
                bitmap[bitmap_offset as usize - 1] |= byte_offset;
            }
        }

        let (free_frame_offset, free_frame_byte_offset) =
            FrameAllocator::offsets(bootstrap_frame_alloc.free().0);

        Self {
            bitmap,
            free_frame_offset,
            free_frame_byte_offset,
        }
    }

    fn offsets(addr: usize) -> (usize, u8) {
        let frame_no = addr / PAGE_SIZE;
        let byte_offset = (frame_no % 8) as u8;
        let offset = (frame_no - byte_offset as usize) / 8;

        (offset, byte_offset)
    }
}

impl FrameAlloc for FrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        let frame_no = (self.free_frame_offset + self.free_frame_byte_offset as usize) * 8;
        let mut addr = frame_no * PAGE_SIZE;
        let frame = Frame::from_physical_address(PhysicalAddress::new(addr));

        addr += PAGE_SIZE;
        let (mut free_frame_offset, mut free_frame_byte_offset) = FrameAllocator::offsets(addr);
        if free_frame_offset >= self.bitmap.len() {
            addr = 0;
            let mut found_free_frame = false;
            while free_frame_offset < self.bitmap.len() {
                if self.bitmap[addr] & free_frame_byte_offset == 0 {
                    found_free_frame = true;
                    break;
                }
                addr += PAGE_SIZE;
                (free_frame_offset, free_frame_byte_offset) = FrameAllocator::offsets(addr);
            }

            if !found_free_frame {
                panic!("OOM: No free frames");
            }
        }

        Some(frame)
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        let (offset, byte_offset) = FrameAllocator::offsets(frame.physical_address().0);
        self.bitmap[offset] = self.bitmap[offset] ^ byte_offset;
    }
}
