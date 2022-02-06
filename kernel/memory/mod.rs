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

    let mut page_mapper = PageMapper::init_kernel_tables();

    let test_frame = Frame { frame_number: 20_000 };
    page_mapper.identity_map(test_frame, &mut frame_allocator);
}

#[derive(Clone, Copy)]
pub struct Page {
    pub page_number: usize
}

impl Page {
    pub fn virtual_address(&self) -> VirtualAddress {
        self.page_number * PAGE_SIZE
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
