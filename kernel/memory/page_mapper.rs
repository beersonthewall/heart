use core::arch::asm;
use super::PAGE_SIZE;
use super::frame_alloc::FrameAllocator;
use super::page::Page;
use super::frame::Frame;
use super::addr::{PhysicalAddress, VirtualAddress};

//const TABLE_SIZE: usize = 512;

// Recurisve page table constants.
const P4_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff7f_bfdf_e000);
const P3_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff7f_bfc0_0000);
const P2_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff7f_8000_0000);
const P1_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff00_0000_0000);

// Page table flags
const PTE_PRESENT: usize = 0b1;
const PTE_READ_WRITE: usize = 0b10;
//const PTE_PROT: usize = 0b100;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum PageTableKind {
    // Page Map Level 4
    PML4,

    // Page Directory Pointer Table
    PDPT,

    // Page Directory
    PD,

    // Page Table
    PT,
}

#[derive(Copy, Clone)]
pub struct Table {
    entries: *mut PageTableEntry,
    kind: PageTableKind,
}

impl Table {
    fn dbg_addr(&self) {
        let k = self.kind;
        let e = self.entries;
        log!("kind: {k:?}; vaddr: {e:?}");
    }

    fn from_virtual_address(address: usize, kind: PageTableKind) -> Self {
        let table_ptr = address as *mut PageTableEntry;

        Self {
            entries: table_ptr,
            kind: kind,
        }
    }

    fn next_table(&mut self, page: Page) -> Table {
        assert!(self.kind != PageTableKind::PT);
        let k = self.kind;
        let table: Table;

        if self.kind == PageTableKind::PML4 {
            table = Table::from_virtual_address(P3_TABLE_BASE.0 | (page.pml4_offset() << 12), PageTableKind::PDPT);
        } else if self.kind == PageTableKind::PDPT {
            table = Table::from_virtual_address(P2_TABLE_BASE.0 | (page.pml4_offset() << 21) | (page.pdpt_offset() << 12), PageTableKind::PD);
        } else {
            table = Table::from_virtual_address(P1_TABLE_BASE.0 | (page.pml4_offset() << 30) | (page.pdpt_offset() << 21) | (page.pd_offset() << 12), PageTableKind::PT);
        }

        return table;
    }

    fn contains(&self, page: Page) -> bool {
        let offset = match self.kind {
            PageTableKind::PML4 => page.pml4_offset(),
            PageTableKind::PDPT => page.pdpt_offset(),
            PageTableKind::PD => page.pd_offset(),
            PageTableKind::PT => page.pt_offset(),
        };

        unsafe {
            return (*(self.entries.offset(offset as isize))).0 != 0;
        }
    }

    fn add_entry(&mut self, page: Page, table_address: PhysicalAddress) {
        let offset: usize = match self.kind {
            PageTableKind::PML4 => page.pml4_offset(),
            PageTableKind::PDPT => page.pdpt_offset(),
            PageTableKind::PD => page.pd_offset(),
            PageTableKind::PT => page.pt_offset(),
        };

        let k = self.kind;
        unsafe {
            // Assert we are not destroying existing mappings.
            let ptr: *mut PageTableEntry = self.entries.add(offset);
            let entry = (table_address.0) | PTE_READ_WRITE | PTE_PRESENT;
            
            log!("adding entry 0x{entry:x} at location {ptr:?}");

            assert!(!(*ptr).is_used());

            core::ptr::write_volatile(ptr, PageTableEntry(entry as u64));
            print_table(self.kind, page);

            asm!("invlpg [{}]", in(reg) ptr, options(nostack, preserves_flags));

            let value = core::ptr::read_volatile(ptr).0;
            assert!(value != 0);
            log!("Value read: 0x{value:x}");
        }
    }
}

pub struct PageMapper {
    root: Table,
}

impl PageMapper {
    pub fn init_kernel_table() -> Self {
        let kernel_pml4: VirtualAddress = P4_TABLE_BASE;
        // The initial kernel_pml4 is identity mapped so the cast from physical (value stored in cr3 is a physical address)
        // to virtual is a-okay. However it may be problematic in the future. I should look into using the recursive entry
        // to generate a virtual address for the root.
        Self {
            root: Table::from_virtual_address(kernel_pml4.0, PageTableKind::PML4),
        }
    }

    pub fn map(&mut self, page: Page, frame: Frame, alloc: &mut FrameAllocator) {
        let sz = core::mem::size_of::<PageTableEntry>();
        let mut current_table = self.root;
        let mut level = Some(PageTableKind::PML4);

        while level.is_some() {
            let l = level.unwrap();
            log!("mapping: {l:?}");
            let ct = current_table.kind;
            log!("Current table: {ct:?}");

            if !current_table.contains(page) {
                let frame = alloc.allocate_frame().expect("[PageMapper.map()] failed to allocate new frame for page table.");
                let physical_address = frame.physical_address();
                let pa = physical_address.0;
                let frameno = frame.frame_number;
                print_table(PageTableKind::PML4, page);
                current_table.add_entry(page, physical_address);

                let lvl = match l {
                    PageTableKind::PML4 => PageTableKind::PDPT,
                    PageTableKind::PDPT => PageTableKind::PD,
                    PageTableKind::PD => PageTableKind::PT,
                    _ => {
                        log!("panic matching level");
                        panic!();
                    },
                };

                // We need to clear that memory.
                let new_table_virtual_address = match level.unwrap() {
                    PageTableKind::PML4 => P3_TABLE_BASE.0 | (page.pml4_offset() << 12),
                    PageTableKind::PDPT => P2_TABLE_BASE.0 | (page.pml4_offset() << 21) | (page.pdpt_offset() << 12),
                    PageTableKind::PD => P1_TABLE_BASE.0 | (page.pml4_offset() << 30) | (page.pdpt_offset() << 21) | (page.pd_offset() << 12),
                    PageTableKind::PT => panic!("Tried to alloc new frame with nowhere to go!"),
                };
                print_table(PageTableKind::PML4, page);
                current_table.dbg_addr();
                log!("clearing newly allocated frame using virtual addr: 0x{new_table_virtual_address:x}");
                unsafe {
                    let mut s = core::slice::from_raw_parts_mut(new_table_virtual_address as *mut u8, PAGE_SIZE);
                    s.fill(0);
                }
                log!("Completed clearing frame");

                log!("Printing newly allocated table");
                print_table(lvl, page);
            }
            log!("next_table");
            current_table = current_table.next_table(page);

            level = match level {
                Some(PageTableKind::PML4) => Some(PageTableKind::PDPT),
                Some(PageTableKind::PDPT) => Some(PageTableKind::PD),
                Some(PageTableKind::PD) => Some(PageTableKind::PT),
                Some(PageTableKind::PT) => None,
                None => None,
            }
        }

        // Add the final page table entry.
        current_table.add_entry(page, PhysicalAddress::new(0x0));
        log!("Finished mapping entry!");
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct PageTableEntry(u64);

impl PageTableEntry {
    fn new() -> Self {
        PageTableEntry(0)
    }

    fn is_used(&self) -> bool {
        self.0 != 0
    }
}

fn print_table(l: PageTableKind, page: Page) {
    unsafe {
        let addr = match l {
            PageTableKind::PML4 => P4_TABLE_BASE.0,
            PageTableKind::PDPT => P3_TABLE_BASE.0 | (page.pml4_offset() << 12),
            PageTableKind::PD => P2_TABLE_BASE.0 | (page.pml4_offset() << 21) | (page.pdpt_offset() << 12),
            PageTableKind::PT => P1_TABLE_BASE.0 | (page.pml4_offset() << 30) | (page.pdpt_offset() << 21) | (page.pd_offset() << 12),
        };

        log!("printing {l:?}: virtual address: 0x{addr:x}");
        let mut is_zero = true;
        for i in 0..512 {
            let virtual_address = (addr as *const usize).add(i);
            let value: usize = *((addr as *const usize).add(i));
            if value != 0 {
                is_zero = false;
                log!("{l:?} entry at offset {i}: 0x{value:x}. address {virtual_address:?}");
            }
        }
        if is_zero {
            log!("{l:?} is empty");
        }

    }
}
