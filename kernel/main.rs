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

// Kernel entrypoint (called by arch/<foo>/start.S)
#[no_mangle]
pub extern "C" fn kmain(multiboot_ptr: usize) {
    extern "C" {
        static kernel_end: u8;
    }

    log!("Hello world! :)");
    let kend: usize;
    unsafe { kend = &kernel_end as *const _ as usize; }
    log!("kernel_end: 0x{:x}", kend);
    log!("multiboot ptr: 0x:{:x}", multiboot_ptr);
    crate::memory::init(multiboot_ptr, kend);
}
