#![feature(allocator_api)]
#![feature(asm_const)]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(ptr_to_from_bits)]
#![no_std]
#![crate_name = "kernel"]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;
#[macro_use]
extern crate lazy_static;
extern crate alloc;
extern crate bit_field;
extern crate pic8259;
extern crate spin;
extern crate kernel_api;

#[cfg(target_arch = "x86_64")]
#[path = "arch/amd64/mod.rs"]
pub mod arch;
pub mod unwind;

mod filesystem;
mod logging;
mod memory;
mod multiboot;
//mod intrusive_list;

use self::arch::memory::PAGE_SIZE;

const KERNEL_BASE: usize = 0xFFFFFFFF80000000;

// Kernel entrypoint (called by arch/<foo>/start.S)
#[no_mangle]
pub extern "C" fn kmain(multiboot_ptr: usize) {
    extern "C" {
        static kernel_end: u8;
    }

    log!("Hello world! :)");
    let kend_vaddr: usize = unsafe { &kernel_end as *const _ as usize };
    let kend_phys_addr = kend_vaddr - KERNEL_BASE;
    // page align heap start
    let bootstrap_frame_alloc_start = kend_phys_addr + PAGE_SIZE - (kend_phys_addr % PAGE_SIZE);
    log!("kendvaddr: {:x}", kend_vaddr);
    memory::init(multiboot_ptr, bootstrap_frame_alloc_start, kend_vaddr);
    filesystem::init();
    arch::interrupt::init();
}
