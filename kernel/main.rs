#![feature(panic_info_message)]
#![no_std]
#![crate_name = "kernel"]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

#[cfg(target_arch = "x86_64")]
#[path = "arch/amd64/mod.rs"]
pub mod arch;
pub mod unwind;

mod logging;
mod memory;
mod multiboot;

use self::memory::PAGE_SIZE;

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
    crate::memory::init(multiboot_ptr, heap_start_physical);
}
