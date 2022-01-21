#![feature(panic_info_message)]
#![no_std]
#![crate_name="kernel"]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

#[cfg(target_arch="x86_64")] #[path="arch/amd64/mod.rs"]
pub mod arch;

pub mod unwind;

mod logging;

// Kernel entrypoint (called by arch/<foo>/start.S)
#[no_mangle]
pub fn kmain()
{
	log!("Hello world! 1={}", 1);
	loop {}
}
