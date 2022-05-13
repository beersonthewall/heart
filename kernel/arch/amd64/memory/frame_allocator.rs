use super::page_mapper::PageMapper;
use super::PAGE_SIZE;
use crate::memory::{
    addr::{PhysicalAddress, VirtualAddress},
    frame::Frame,
    page::Page,
    FrameAllocatorAPI,
};
use crate::multiboot::{MMapEntryType, MultibootInfo};
use spin::mutex::Mutex;

pub struct BootstrapFrameAllocator {
    start: PhysicalAddress,
    free: Frame,
}

impl BootstrapFrameAllocator {
    pub fn new(start: PhysicalAddress) -> Self {
        Self {
            start,
            free: Frame::from_physical_address(start),
        }
    }

    fn start(&self) -> PhysicalAddress {
        self.start
    }

    fn free(&self) -> PhysicalAddress {
        self.free.physical_address()
    }
}

impl FrameAllocatorAPI for BootstrapFrameAllocator {
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

pub struct FrameAllocatorInner<'a> {
    bitmap: &'a mut [u8],
    free_frame_offset: usize,
    free_frame_byte_offset: u8,
}

impl<'a> FrameAllocatorInner<'a> {
    pub fn new(
        mut bootstrap_frame_alloc: BootstrapFrameAllocator,
        info: &MultibootInfo,
        page_mapper: &mut PageMapper,
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
            let frame = bootstrap_frame_alloc.allocate_frame().unwrap();

            // The bootstrap frame allcator starts at kernel end which may or may not be identity
            // mapped in start.S. We map a single 2MB page which is supposed to cover the kernel,
            // but will cover the frames allocated by the bootstrap allocator if the kernel < 2MB.
            let page = Page::from_virtual_address(VirtualAddress::new(frame.physical_address().0));
            if !page_mapper.is_mapped(page) {
                page_mapper
                    .map(page, frame, &mut bootstrap_frame_alloc)
                    .expect("Failure mapping page in frame allocator creation.");
            }
        }

        let ptr = bitmap_start_frame.physical_address().0 as *mut u8;
        // FIXME need to map ptr, might be id mapped right now though? just because kernel < 2MB and
        // we mapped 2 MB in start.S (at least for amd64 arch).
        let bitmap = unsafe { core::slice::from_raw_parts_mut(ptr, bitmap_sz) };

        // FIXME: detect & mark regions not in the memory map as reserved.
        // We won't necessarily have all existing memory in the map.
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
                let (bitmap_offset, byte_offset) = Self::offsets(addr);
                bitmap[bitmap_offset as usize - 1] |= byte_offset;
            }
        }

        // Mark frames allocated by bootstrap frame allocator as used.
        for frame_addr in
            (bootstrap_frame_alloc.start().0..bootstrap_frame_alloc.free().0).step_by(PAGE_SIZE)
        {
            let (bitmap_offset, byte_offset) = Self::offsets(frame_addr);
            bitmap[bitmap_offset as usize - 1] |= byte_offset;
        }

        let (free_frame_offset, free_frame_byte_offset) =
            Self::offsets(bootstrap_frame_alloc.free().0);

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

impl FrameAllocatorAPI for FrameAllocatorInner<'_> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        let frame_no = (self.free_frame_offset + self.free_frame_byte_offset as usize) * 8;
        let mut addr = frame_no * PAGE_SIZE;
        let frame = Frame::from_physical_address(PhysicalAddress::new(addr));

        addr += PAGE_SIZE;
        let (mut free_frame_offset, mut free_frame_byte_offset) = Self::offsets(addr);
        if free_frame_offset >= self.bitmap.len() {
            addr = 0;
            let mut found_free_frame = false;
            while free_frame_offset < self.bitmap.len() {
                if self.bitmap[addr] & free_frame_byte_offset == 0 {
                    found_free_frame = true;
                    break;
                }
                addr += PAGE_SIZE;
                (free_frame_offset, free_frame_byte_offset) = Self::offsets(addr);
            }

            if !found_free_frame {
                panic!("OOM: No free frames");
            }
        }

        Some(frame)
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        let (offset, byte_offset) = Self::offsets(frame.physical_address().0);
        self.bitmap[offset] = self.bitmap[offset] ^ byte_offset;
    }
}

pub struct FrameAllocator<'a> {
    pub inner: Mutex<Option<FrameAllocatorInner<'a>>>,
}

impl<'a> FrameAllocator<'a> {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

impl<'a> FrameAllocatorAPI for FrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(ref mut fa) = *self.inner.lock() {
            fa.allocate_frame()
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        if let Some(ref mut fa) = *self.inner.lock() {
            fa.deallocate_frame(frame);
        }
    }
}
