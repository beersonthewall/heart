use super::addr::VirtualAddress;
use super::frame::Frame;
use super::frame_alloc::FrameAlloc;
use super::frame_alloc::FrameAllocator;
use super::page::Page;
use super::page_table::{PageTableEntry, Table, PTE_PRESENT, PTE_WRITE};
use super::PagingError;

// Recursive page table constants.
// Note: the recursive entry is at index 510.
const P4_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff7f_bfdf_e000);
const RECURSIVE_INDEX: usize = 510;

pub struct PageMapper<'a> {
    root: &'a mut Table,
}

impl<'a> PageMapper<'a> {
    pub fn init_kernel_table() -> Self {
        Self {
            root: Table::from_virtual_address(P4_TABLE_BASE),
        }
    }

    #[allow(dead_code)]
    fn print_table(table: &Table) {
        let mut nonzero = false;
        for i in 0..512 {
            let entry = &table[i];
            let val = entry.entry();
            if entry.is_used() {
                nonzero = true;
                log!("entry {i}: {val:x}");
            }
        }
        if !nonzero {
            log!("table empty");
        }
    }

    fn next_table<FA>(entry: &mut PageTableEntry, next: Page, alloc: &mut FA) -> &'a mut Table
    where
        FA: FrameAlloc,
    {
        if !entry.is_used() {
            if let Some(frame) = alloc.allocate_frame() {
                entry.set_frame(frame, PTE_WRITE | PTE_PRESENT);
            } else {
                panic!("Failed to allocate frame for next_table.");
            }
        }

        let vaddr = next.virtual_address().0;
        log!("table vaddr: {vaddr:x}");
        let table = unsafe { &mut *(next.virtual_address().0 as *mut Table) };
        return table;
    }

    pub fn map<FA>(&mut self, page: Page, frame: Frame, alloc: &mut FA) -> Result<(), PagingError>
    where
        FA: FrameAlloc,
    {
        log!("writing pml4 entry");
        let pdpt_page = recursive_page(
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            page.pml4_offset(),
        );
        let pdpt = PageMapper::next_table(&mut self.root[page.pml4_offset()], pdpt_page, alloc);

        log!("writing pdpt entry");
        let pd_page = recursive_page(
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            page.pml4_offset(),
            page.pdpt_offset(),
        );
        let pd = PageMapper::next_table(&mut pdpt[page.pdpt_offset()], pd_page, alloc);

        log!("writing pd entry");
        let pt_page = recursive_page(
            RECURSIVE_INDEX,
            page.pml4_offset(),
            page.pdpt_offset(),
            page.pd_offset(),
        );
        let pt = PageMapper::next_table(&mut pd[page.pd_offset()], pt_page, alloc);

        log!("writing pt entry");
        let entry = &mut pt[page.pt_offset()];
        entry.set_frame(frame, PTE_WRITE | PTE_PRESENT);

        Ok(())
    }

    pub fn unmap<FA>(&mut self, page: Page, frame: Frame, alloc: &mut FA) -> Result<(), PagingError>
    where
        FA: FrameAlloc,
    {
        let pml4_entry = &mut self.root[page.pml4_offset()];
        assert!(pml4_entry.is_used());

        let pdpt_page = recursive_page(
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            page.pml4_offset(),
        );
        let pdpt_table = PageMapper::next_table(pml4_entry, pdpt_page, alloc);
        let pdpt_entry = &mut pdpt_table[page.pdpt_offset()];
        assert!(pdpt_entry.is_used());

        let pd_page = recursive_page(
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            page.pml4_offset(),
            page.pdpt_offset(),
        );
        let pd_table = PageMapper::next_table(pdpt_entry, pd_page, alloc);
        let pd_entry = &mut pd_table[page.pd_offset()];
        assert!(pd_entry.is_used());

        let pt_page = recursive_page(
            RECURSIVE_INDEX,
            page.pml4_offset(),
            page.pdpt_offset(),
            page.pd_offset(),
        );
        let pt_table = PageMapper::next_table(pd_entry, pt_page, alloc);
        let pt_entry = &mut pt_table[page.pt_offset()];
        assert!(pt_entry.is_used());
        assert!(frame == pt_entry.frame());

        Ok(())
    }

    pub fn is_mapped(&self, page: Page) -> bool {
        if !self.root[page.pml4_offset()].is_used() {
            return false;
        }

        let pdpt_page = recursive_page(
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            page.pml4_offset(),
        );
        let pdpt = unsafe { &mut *(pdpt_page.virtual_address().0 as *mut Table) };

        if !pdpt[page.pdpt_offset()].is_used() {
            return false;
        }

        let pd_page = recursive_page(
            RECURSIVE_INDEX,
            RECURSIVE_INDEX,
            page.pml4_offset(),
            page.pdpt_offset(),
        );
        let pd = unsafe { &mut *(pd_page.virtual_address().0 as *mut Table) };

        if !pd[page.pd_offset()].is_used() {
            return false;
        }

        let pt_page = recursive_page(
            RECURSIVE_INDEX,
            page.pml4_offset(),
            page.pdpt_offset(),
            page.pd_offset(),
        );
        let pt = unsafe { &mut *(pt_page.virtual_address().0 as *mut Table) };

        return pt[page.pt_offset()].is_used();
    }
}

#[inline]
fn recursive_page(pml4_index: usize, pdpt_index: usize, pd_index: usize, pt_index: usize) -> Page {
    let addr: usize = (pml4_index << 39) | (pdpt_index << 30) | (pd_index << 21) | (pt_index << 12);
    log!("new recursive page: {addr:x}");
    Page::from_virtual_address(VirtualAddress(addr))
}
