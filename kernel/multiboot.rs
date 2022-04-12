use core::iter::Iterator;
use core::mem::size_of;

pub struct MultibootInfo {
    raw_data: *const u8,
}

/// Working from the manuals found at: https://www.gnu.org/software/grub/manual/multiboot/.
#[allow(dead_code)]
impl MultibootInfo {
    pub fn new(multiboot_ptr: usize) -> Self {
        MultibootInfo {
            raw_data: multiboot_ptr as *const u8,
        }
    }

    pub fn flags(&self) -> u32 {
        unsafe { *(self.raw_data as *const u32) }
    }

    pub fn mem_lower(&self) -> u32 {
        if !self.flag_is_set(0b1) {
            return 0;
        }

        unsafe { *(self.raw_data.offset(4) as *const u32) }
    }

    pub fn mem_upper(&self) -> u32 {
        if !self.flag_is_set(0b1) {
            return 0;
        }

        unsafe { *(self.raw_data.offset(8) as *const u32) }
    }

    pub fn mmap_iter(&self) -> MMapIter {
        MMapIter::new(self.mmap_addr() as *const u8, self.mmap_length())
    }

    pub fn mmap_length(&self) -> u32 {
        if !self.flag_is_set(1 << 6) {
            return 0;
        }
        unsafe { *(self.raw_data.offset(44) as *const u32) }
    }

    pub fn mmap_addr(&self) -> u32 {
        if !self.flag_is_set(1 << 6) {
            return 0;
        }

        unsafe { *(self.raw_data.offset(48) as *const u32) }
    }

    #[inline]
    fn flag_is_set(&self, flag: u32) -> bool {
        (self.flags() & flag) != 0
    }
}

#[derive(Debug)]
pub enum MMapEntryType {
    Available,
    Reserved,
    ACPI,
    PreserveOnHibernate,
    DefectiveRAM,
}

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct MMapEntry {
    pub size: u32,
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
}

impl MMapEntry {
    pub fn size(&self) -> u32 {
        unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.size)) }
    }

    pub fn base_addr(&self) -> u64 {
        unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.base_addr)) }
    }

    pub fn length(&self) -> u64 {
        unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.length)) }
    }

    pub fn entry_type(&self) -> MMapEntryType {
        let value;
        unsafe { value = core::ptr::read_unaligned(core::ptr::addr_of!(self.entry_type)) }
        match value {
            1 => MMapEntryType::Available,
            3 => MMapEntryType::ACPI,
            4 => MMapEntryType::PreserveOnHibernate,
            5 => MMapEntryType::DefectiveRAM,
            _ => MMapEntryType::Reserved,
        }
    }
}

pub struct MMapIter {
    start: *const u8,
    length: u32,
    current_offset: isize,
}

impl MMapIter {
    fn new(start: *const u8, length: u32) -> Self {
        Self {
            start: start,
            length: length,
            current_offset: 0,
        }
    }
}

impl Iterator for MMapIter {
    type Item = MMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset >= self.length as isize {
            return None;
        }

        let entry;
        unsafe {
            entry = *(self.start.offset(self.current_offset) as *const MMapEntry);
        }
        self.current_offset =
            self.current_offset + (entry.size() as isize) + (size_of::<u32>() as isize);
        Some(entry)
    }
}
