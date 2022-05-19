mod idt;

use core::arch::asm;
use idt::{InterruptDescriptorTable, InterruptHandlerFn};
use spin::mutex::Mutex;

const IDT_SIZE: usize = 256;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.set_handler(0, divide_by_zero_handler);
        idt
    };
}

pub fn init() {
    /*
    - Make space for the interrupt descriptor table -> done, added static array
    - Tell the CPU where that space is (see GDT Tutorial: lidt works the very same way as lgdt)
    - Tell the PIC that you no longer want to use the BIOS defaults (see Programming the PIC chips)
    - Write a couple of ISR handlers (see Interrupt Service Routines) for both IRQs and exceptions
    - Put the addresses of the ISR handlers in the appropriate descriptors (in Interrupt Descriptor Table)
    - Enable all supported interrupts in the IRQ mask (of the PIC)
     */

    log!("load idt");
    IDT.load();
    log!("Here it goes....");
    divide_by_zero();
    log!("Didn't crash");
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx", out("ax") _, out("dx") _);
    }
}

#[naked]
extern "C" fn do_nothing() -> ! {
    unsafe {
        asm!("iret", options(noreturn));
    }
}

extern "C" fn divide_by_zero_handler() -> ! {
    log!("EXCEPTION: divide by zero");
    loop {}
}
