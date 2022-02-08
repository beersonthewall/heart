mod frame_alloc;
mod page_mapper;

use core::mem::size_of;

use crate::multiboot::MultibootInfo;
use frame_alloc::FrameAllocator;
use page_mapper::PageMapper;

const PAGE_SIZE: usize = 4096;

pub fn init(multiboot_addr: usize) {
    // Identity map kernel and multiboot information struct, but notably *not* the maps
    // referenced by the multiboot info struct. We can't read it yet so we don't know
    // where they are located in physical memory.

    let kernel_virtual_range = VirtualAddressRange::new(0xFFFFFFFF80000000, 4_000_000);
    let kernel_physical_range = PhysicalAddressRange::new(0xFFFFFFFF80000000, 4_000_000);
    let multiboot_virtual_range =
        VirtualAddressRange::new(multiboot_addr, size_of::<MultibootInfo>());
    let multiboot_physical_range =
        PhysicalAddressRange::new(multiboot_addr, size_of::<MultibootInfo>());

    let mut frame_allocator = FrameAllocator::new(
        multiboot_physical_range.base,
        multiboot_physical_range.base + multiboot_physical_range.size,
        kernel_physical_range.base,
        kernel_physical_range.base + kernel_physical_range.size,
        // Start immediately after the kernel
        kernel_physical_range.base + kernel_physical_range.size + 1,
    );

    let mut page_mapper = PageMapper::init_kernel_table();

    let test_frame = Frame { frame_number: 0x55_55_BB_00_BB_BB_BB_BB / PAGE_SIZE };
    let test_page = Page { page_number: 0xAA_AA_AA_AA_AA_AA_AA_AA / PAGE_SIZE };
    let addr: u64 = 0b1111111111111111_111111110_111111110_111111110_111111110_000000000000;
    let addr = addr as *const u64;
    unsafe {
        let addr = addr.add(510);
        let value = *addr;
        log!("VALUE: 0x{value:X}");
    }
    page_mapper.map(test_page, test_frame, &mut frame_allocator);
}

#[derive(Clone, Copy)]
pub struct Page {
    pub page_number: usize
}

impl Page {
    pub fn virtual_address(&self) -> VirtualAddress {
        self.page_number * PAGE_SIZE
    }

    pub fn pml4_offset(&self) -> usize {
        (self.virtual_address() >> 39) & 0x1FF
    }

    pub fn pdpt_offset(&self) -> usize {
        (self.virtual_address() >> 30) & 0x1FF
    }

    pub fn pd_offset(&self) -> usize {
        (self.virtual_address() >> 21) & 0x1FF
    }

    pub fn pt_offset(&self) -> usize {
        (self.virtual_address() >> 12) & 0x1FF
    }

    pub fn physical_page_offset(&self) -> usize {
        (self.virtual_address() >> 0) & 0x1FF
    }
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub frame_number: usize,
}

impl Frame {
    pub fn from_physical_address(addr: &PhysicalAddress) -> Self {
        Self {
            frame_number: *addr / PAGE_SIZE,
        }
    }

    pub fn physical_address(&self) -> PhysicalAddress {
        self.frame_number * PAGE_SIZE
    }
}

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

#[derive(Copy, Clone)]
pub struct PhysicalAddressRange {
    pub base: PhysicalAddress,
    pub size: usize,
}

impl PhysicalAddressRange {
    fn new(base: PhysicalAddress, size: usize) -> Self {
        Self { base, size }
    }
}

#[derive(Copy, Clone)]
pub struct VirtualAddressRange {
    pub base: VirtualAddress,
    pub size: usize,
}

impl VirtualAddressRange {
    fn new(base: VirtualAddress, size: usize) -> Self {
        Self { base, size }
    }
}
