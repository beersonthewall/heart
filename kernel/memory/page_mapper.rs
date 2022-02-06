use core::arch::asm;
use core::ptr::{NonNull, addr_of};

use crate::memory::{
    frame_alloc::FrameAllocator, Frame, Page, PhysicalAddress, PhysicalAddressRange,
    VirtualAddress, VirtualAddressRange, PAGE_SIZE,
};

const TABLE_SIZE: usize = 512;

struct Table {
    data: [usize; TABLE_SIZE],
    next: Option<NonNull<Table>>,
}

impl Table {
    fn new() -> Self {
        Self {
            data: [0; TABLE_SIZE],
            next: None,
        }
    }
}

pub struct PageMapper {
    pml4: PageMapLevel4,
}

impl PageMapper {
    pub fn init_kernel_tables() -> Self {
        let mut pml4_physical_address: usize = 0;
        let page_map_l4: PageMapLevel4;

        log!("[init_kernel_tables] Loading kernel page table from cr3.");

        unsafe {
            asm!("mov {}, cr3", out(reg) pml4_physical_address);
        }
        // read() copies the existing pml4, changes will only be applied once we write the
        // address of this new pml4 to cr3.
        unsafe {
            page_map_l4 = core::ptr::read(pml4_physical_address as *mut PageMapLevel4);
        }
        let pml4 = page_map_l4;

        Self { pml4 }
    }

    pub fn identity_map(&mut self, frame: Frame, alloc: &mut FrameAllocator) {
        // TODO how to mark this as allocated in the virtual address space?
        log!("id map start");
        let page = Page { page_number: frame.frame_number };
        self.map(page, frame, alloc);
        write_cr3(addr_of!(self.pml4) as usize);
        log!("id map");
    }

    pub fn map(&mut self, page: Page, frame: Frame, alloc: &mut FrameAllocator) {
        let virtual_address = page.virtual_address();
        let pml4_offset = (virtual_address >> 39) & 0x1FF;
        let pdpt: &mut PageDirectoryPointerTable;

        log!("[page mapper] before pdpt cast");
        if self.pml4.is_present(pml4_offset) {
            log!("[page mapper] before existing pdpt");
            let pdpt_physical_address =
                (self.pml4.entries[pml4_offset] >> 11) & 0x07_FF_FF_FF_FF_FF_FF;
            unsafe {
                pdpt = &mut *(pdpt_physical_address as *mut PageDirectoryPointerTable);
            };
            log!("[page mapper] after existing pdpt");
        } else {
            log!("[page mapper] before alloc new pdpt");
            let frame = alloc
                .allocate_frame()
                .expect("[page_mapper.map()] failed to allocate new phsyical frame.");
            self.identity_map(frame, alloc);
            unsafe {
                pdpt = &mut *(frame.physical_address() as *mut PageDirectoryPointerTable);
            }
            log!("[page mapper] after alloc new pdpt");
        }
        log!("[page mapper] after pdpt cast");

        pdpt.map(virtual_address, frame, alloc, self);

        log!("[page mapper] after recursive map");
        // 0x3 = present + writable. Then put the 51 bit address at the correct spot. bits 12-51x
        let new_entry = 0x3 | (((pdpt as *const PageDirectoryPointerTable) as usize & (0x07_FF_FF_FF_FF_FF_FF)) << 11);
        pdpt.entries[pml4_offset] = new_entry;
    }
}

/// Writes the provided value into CR3 register. Used for configuring new page tables.
pub fn write_cr3(value: usize) {
    unsafe {
        asm!("mov cr3, {}", in(reg) value);
    }
}

#[repr(C)]
struct PageMapLevel4 {
    entries: [usize; 512],
}

impl PageMapLevel4 {
    fn is_present(&self, offset: usize) -> bool {
        self.entries[offset] & 0x1 == 0x1
    }
}

#[repr(C)]
struct PageDirectoryPointerTable {
    entries: [usize; 512],
}

impl PageDirectoryPointerTable {
    fn is_present(&self, offset: usize) -> bool {
        self.entries[offset] & 0x1 == 0x1
    }

    fn map(&mut self, page: VirtualAddress, frame: Frame, alloc: &mut FrameAllocator, mapper: &mut PageMapper) {
        let pdpt_offset = (page >> 29) & 0x1FF;
        let pd: &mut PageDirectory;

        if self.is_present(pdpt_offset) {
            log!("[pdpt] before exsting pd");
            let pd_ptr = (self.entries[pdpt_offset] >> 11) & 0x07_FF_FF_FF_FF_FF_FF;
            unsafe { pd = &mut *(pd_ptr as *mut PageDirectory); }
            log!("[pdpt] after exsting pd");
        } else {
            log!("[pdpt] before new pd");
            let frame = alloc.allocate_frame().expect("[pdpt.map()] failed to allocate new phsyical frame.");
            unsafe { pd = &mut *(frame.physical_address() as *mut PageDirectory); }
            log!("[pdpt] identity mapping new frame");
            mapper.identity_map(frame, alloc);
            log!("[pdpt] after new pd");
        }

        log!("[pdpt] before map call");
        pd.map(page, frame, alloc, mapper);
        log!("[pdpt] after map");
        // 0x3 = present + writable. Then put the 51 bit address at the correct spot. bits 12-51x
        let new_entry = 0x3 | (( (pd as *const PageDirectory) as usize & (0x07_FF_FF_FF_FF_FF_FF)) << 11);
        self.entries[pdpt_offset] = new_entry;
    }
}

#[repr(C)]
struct PageDirectory {
    entries: [usize; 512],
}

impl PageDirectory {

    fn is_present(&self, offset: usize) -> bool {
        self.entries[offset] & 0x1 == 0x1
    }
    
    fn map(&mut self, page: VirtualAddress, frame: Frame, alloc: &mut FrameAllocator, mapper: &mut PageMapper) {
        log!("[pd] before pt off");
        let pt_offset = (page >> 20) & 0x1FF;
        log!("[pd] after pt off");
        let pt: &mut PageTable;

        log!("[pd] before check");
        if self.is_present(pt_offset) {
            log!("[pd] before existing pt");
            let mut c = 0;
            for i in 0..20000 {
                c += i + 1
            }
            log!("{c}");
            let pt_ptr = (self.entries[pt_offset] >> 11) & 0x07_FF_FF_FF_FF_FF_FF;
            unsafe { pt = &mut *(pt_ptr as *mut PageTable); }
            log!("[pd] after existing pt");
        } else {
            log!("[pd] before new pt");
            let mut c = 0;
            for i in 0..20000 {
                c += i + 1
            }
            log!("{c}");
            let frame = alloc.allocate_frame().expect("[pd.map()] failed to allocate new phsyical frame.");
            unsafe { pt = &mut *(frame.physical_address() as *mut PageTable); }
            log!("[pd] id mapping new frame");
            mapper.identity_map(frame, alloc);
            log!("[pd] after new pt");
        }

        log!("?");
        pt.map(page, frame);

        // 0x3 = present + writable. Then put the 51 bit address at the correct spot. bits 12-51
        let new_entry = 0x3 | (((pt as *const PageTable) as usize & (0x07_FF_FF_FF_FF_FF_FF)) << 11);
        self.entries[pt_offset] = new_entry;
    }
}

#[repr(C)]
struct PageTable {
    entries: [usize; 512],
}

impl PageTable {
    fn is_present(&self, offset: usize) -> bool {
        self.entries[offset] & 0x1 == 0x1
    }

    fn map(&mut self, page: VirtualAddress, frame: Frame) {
        log!("[pt] new entry");
        let pt_offset = (page >> 11) & 0x1FF;
        // 0x3 = present + writable. Then put the 51 bit address at the correct spot. bits 12-51
        let new_entry = 0x3 | ((frame.physical_address() & (0x07_FF_FF_FF_FF_FF_FF)) << 11);
        self.entries[pt_offset] = new_entry;
        log!("[pt] done");
    }
}
