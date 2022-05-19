use bit_field::BitField;
use core::arch::asm;

pub type InterruptHandlerFn = extern "C" fn() -> !;

#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}

pub struct InterruptDescriptorTable([IDTEntry; 16]);

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self([IDTEntry::missing(); 16])
    }

    pub fn set_handler(&mut self, vector: usize, handler: InterruptHandlerFn) -> &mut EntryOptions {
        self.0[vector] = IDTEntry::new(cs(), handler);
        &mut self.0[vector].options
    }

    pub fn load(&self) {
        use core::mem::size_of;
        let ptr = DescriptorTablePointer {
            limit: (size_of::<Self>() - 1) as u16,
            base: self as *const _ as u64,
        };

        unsafe {
            asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
        }
    }
}

/// The IDT Entries are a little odd. The interrupt service routine handler
/// pointer is broken up into three parts: isr_low, isr_mid, and isr_high.
/// isr_low is [0:15], isr_mid is [16:32], and isr_high is [32:63].
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct IDTEntry {
    isr_low: u16,
    gdt_selector: u16,
    options: EntryOptions,
    isr_mid: u16,
    isr_high: u32,
    reserved: u32,
}

impl IDTEntry {
    fn missing() -> Self {
        Self {
            isr_low: 0,
            gdt_selector: 0,
            options: EntryOptions::minimal(),
            isr_mid: 0,
            isr_high: 0,
            reserved: 0,
        }
    }

    fn new(gdt_selector: u16, handler_addr: InterruptHandlerFn) -> Self {
        let handler_bits = handler_addr as u64;
        Self {
            isr_low: handler_bits as u16,
            gdt_selector: gdt_selector,
            options: EntryOptions::new(),
            isr_mid: (handler_bits >> 16) as u16,
            isr_high: (handler_bits >> 32) as u32,
            reserved: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct EntryOptions(u16);

impl EntryOptions {
    fn minimal() -> Self {
        let mut options = 0;
        options.set_bits(9..12, 0b111); // 'must-be-one' bits
        EntryOptions(options)
    }

    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        let mut value = self.0;
        value.set_bit(15, present);
        self.0 = value;
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        let mut value = self.0;
        value.set_bit(8, !disable);
        self.0 = value;
        self
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        let mut value = self.0;
        value.set_bits(13..15, dpl);
        self.0 = value;
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        let mut value = self.0;
        value.set_bits(0..3, index);
        self.0 = value;
        self
    }
}

fn cs() -> u16 {
    let segment: u16;
    unsafe {
        asm!(concat!("mov {0:x}, ", "cs"), out(reg) segment, options(nomem, nostack, preserves_flags));
    }
    log!("segment: {:x}", segment);
    segment
}
