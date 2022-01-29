mod frame_alloc;
mod page_mapper;

use core::mem::size_of;

use crate::multiboot::MultibootInfo;
use frame_alloc::FrameAllocator;
use page_mapper::{PageMapper, flush_tlb};

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

    let mut page_mapper = PageMapper::new();
    log!("Mapping new kernel page table.");
    page_mapper.map(kernel_virtual_range, kernel_physical_range, &mut frame_allocator);
    page_mapper.map(multiboot_virtual_range, multiboot_physical_range, &mut frame_allocator);
    page_mapper.write_cr3();

    // TODO should map auto-flush or do we need to expose this as part of the API?
    flush_tlb(multiboot_virtual_range);
    flush_tlb(kernel_virtual_range);

    // TODO who is going to hold on to the kernel's page_mapper & frame alloc?
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

/*let multiboot_info: &MultibootInfo;
unsafe { multiboot_info = &*(multiboot_addr as *const MultibootInfo); }
let frame_alloc = FrameAllocator::new(&multiboot_info, multiboot_addr);*/
