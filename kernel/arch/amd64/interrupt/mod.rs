#[macro_use]
mod handlers;
mod idt;

use core::arch::asm;
use handlers::{breakpoint_handler, divide_by_zero_handler};
use idt::{InterruptDescriptorTable, InterruptHandlerFn};
use spin::mutex::Mutex;

const IDT_SIZE: usize = 256;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(3, isr!(breakpoint_handler));
        idt
    };
}

pub fn init() {
    IDT.load();

    unsafe { asm!("int3", options(nomem, nostack)); }
    log!("We made it back :)");
    divide_by_zero();
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx", out("ax") _, out("dx") _);
    }
}
