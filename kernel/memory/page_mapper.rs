use core::arch::asm;
use core::ptr::{addr_of, NonNull};
use core::slice::from_raw_parts;

use crate::memory::{
    frame_alloc::FrameAllocator, Frame, Page, PhysicalAddress, PhysicalAddressRange,
    VirtualAddress, VirtualAddressRange, PAGE_SIZE,
};

const TABLE_SIZE: usize = 512;

// Recurisve page table constants.
const P4_TABLE_BASE: usize = 0b1111111111111111_111111110_111111110_111111110_111111110_000000000000; //(0xFF_FF << 48) | (0x1FE << 38) | (0x1FE << 29) | (0x1FE << 20) | (0x1FE << 11);
const P3_TABLE_BASE: usize = 0b1111111111111111_111111110_111111110_111111110_000000000_000000000000; //(0xFF_FF << 48) | (0x1FE << 38) | (0x1FE << 29) | (0x1FE << 20);
const P2_TABLE_BASE: usize = 0b1111111111111111_111111110_111111110_000000000_000000000_000000000000; //(0xFF_FF << 48) | (0x1FE << 38) | (0x1FE << 29);
const P1_TABLE_BASE: usize = 0b1111111111111111_111111110_000000000_000000000_000000000_000000000000; //(0xFF_FF << 48) | 0x1FE << 38;

// Page table flags
const PTE_PRESENT: usize = 0b1;
const PTE_READ_WRITE: usize = 0b10;
const PTE_PROT: usize = 0b100;

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
    entries: *mut Entry,
    kind: PageTableKind,
}

impl Table {
    fn new(page: Page, kind: PageTableKind) -> Self {
        Table::from_virtual_address(page.virtual_address(), kind)
    }

    fn from_virtual_address(address: VirtualAddress, kind: PageTableKind) -> Self {
        let table_ptr = address as *mut Entry;

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
            table = Table::from_virtual_address(P3_TABLE_BASE | (page.pml4_offset() << 12), PageTableKind::PDPT);
        } else if self.kind == PageTableKind::PDPT {
            table = Table::from_virtual_address(P2_TABLE_BASE | (page.pml4_offset() << 21) | (page.pdpt_offset() << 12), PageTableKind::PD);
        } else {
            table = Table::from_virtual_address(P1_TABLE_BASE | (page.pml4_offset() << 30) | (page.pdpt_offset() << 21) | (page.pd_offset() << 12), PageTableKind::PT);
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
            return *(self.entries.offset(offset as isize)) != 0;
        }
    }

    fn add_entry(&mut self, page: Page, table_address: PhysicalAddress) {
        let offset = match self.kind {
            PageTableKind::PML4 => page.pml4_offset(),
            PageTableKind::PDPT => page.pdpt_offset(),
            PageTableKind::PD => page.pd_offset(),
            PageTableKind::PT => page.pt_offset(),
        };
        let k = self.kind;
        unsafe {
            // Assert we are not destroying existing mappings.
            assert!(*self.entries.offset(offset as isize) == 0);
            *(self.entries.offset(offset as isize)) = table_address | PTE_READ_WRITE | PTE_PRESENT;
            let new_value = *(self.entries.offset(offset as isize));
            let k = self.kind;
            log!("add value: 0x{new_value:x} at offset 0x{offset:x} in level: {k:?}");
        }
    }
}

pub struct PageMapper {
    root: Table,
}

impl PageMapper {
    pub fn init_kernel_table() -> Self {
        let low_pdpt_ptr = P3_TABLE_BASE as *const usize;
        unsafe {
            let entry = *low_pdpt_ptr;
        }
        let kernel_pml4: VirtualAddress = P4_TABLE_BASE;
        // The initial kernel_pml4 is identity mapped so the cast from physical (value stored in cr3 is a physical address)
        // to virtual is a-okay. However it may be problematic in the future. I should look into using the recursive entry
        // to generate a virtual address for the root.
        Self {
            root: Table::from_virtual_address(kernel_pml4, PageTableKind::PML4),
        }
    }

    pub fn map(&mut self, page: Page, frame: Frame, alloc: &mut FrameAllocator) {
        let mut current_table = self.root;
        let mut level = Some(PageTableKind::PML4);

        while level.is_some() {
            log!("mapping: {level:?}");

            if !current_table.contains(page) {
                let frame = alloc.allocate_frame().expect("[PageMapper.map()] failed to allocate new frame for page table.");
                let physical_address = frame.physical_address();
                log!("Allocating physical address: 0x{physical_address:x}");
                current_table.add_entry(page, physical_address);

                // We need to clear that memory.
                let new_table_virtual_address = match level.unwrap() {
                    PageTableKind::PML4 => P3_TABLE_BASE,
                    PageTableKind::PDPT => P2_TABLE_BASE | (page.pml4_offset() << 12),
                    PageTableKind::PD => P1_TABLE_BASE | (page.pml4_offset() << 21) | (page.pdpt_offset() << 12),
                    PageTableKind::PT => panic!("Tried to alloc new frame with nowhere to go!"),
                };
                unsafe {
                    let mut s = core::slice::from_raw_parts_mut(new_table_virtual_address as *mut u8, PAGE_SIZE);
                    s.fill(0);
                }
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
        current_table.add_entry(page, 0x0);
        log!("Finished mapping entry!");
    }
}

/// Writes the provided value into CR3 register. Used for configuring new page tables.
pub fn write_cr3(value: usize) {
    unsafe {
        asm!("mov cr3, {}", in(reg) value);
    }
}

type Entry = usize;

trait PageTableEntry {
    fn exists(self) -> bool;
}
impl PageTableEntry for Entry {
    fn exists(self) -> bool {
        self != 0
    }
}
