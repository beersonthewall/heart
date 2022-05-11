#![feature(allocator_api)]
#![feature(default_alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(ptr_to_from_bits)]
#![no_std]
#![crate_name = "kernel"]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

#[macro_use]
extern crate alloc;
extern crate spin;

#[cfg(target_arch = "x86_64")]
#[path = "arch/amd64/mod.rs"]
pub mod arch;
pub mod unwind;

mod logging;
mod memory;
mod multiboot;

use self::arch::memory::PAGE_SIZE;

const KERNEL_BASE: usize = 0xFFFFFFFF80000000;

// Kernel entrypoint (called by arch/<foo>/start.S)
#[no_mangle]
pub extern "C" fn kmain(multiboot_ptr: usize) {
    extern "C" {
        static kernel_end: u8;
    }

    log!("Hello world! :)");
    let kend_vaddr: usize;
    unsafe {
        kend_vaddr = &kernel_end as *const _ as usize;
    }
    let kend_phys_addr = kend_vaddr - KERNEL_BASE;
    // page align heap start
    let heap_start_physical = kend_phys_addr + PAGE_SIZE - (kend_phys_addr % PAGE_SIZE);
    memory::init(multiboot_ptr, heap_start_physical);
    use alloc::vec::Vec;

    // Test the linked_list_allocator by allocating a larger size than the biggest slab.
    let mut nums: Vec<usize> = Vec::with_capacity(1024);
    for i in 0..1024 {
        nums.push(i);
    }
    log!("{:?}", nums);
}
