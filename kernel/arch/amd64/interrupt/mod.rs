#[macro_use]
mod handlers;
mod idt;

use core::arch::asm;
use handlers::{
    alignment_check_handler, bound_range_handler, breakpoint_handler, control_protection_handler,
    debug_handler, device_not_available_handler, divide_by_zero_handler, double_fault_handler,
    general_protection_handler, hypervisor_injection_handler, invalid_opcode_handler,
    invalid_tss_handler, machine_check_handler, non_maskable_handler, overflow_handler,
    page_fault_handler, security_handler, segment_not_present_handler, simd_floating_point_handler,
    stack_handler, vmm_communication_handler, x87_floating_point_handler,
};
use idt::{InterruptDescriptorTable, InterruptHandlerFn};
use spin::mutex::Mutex;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(1, handler!(debug_handler));
        idt.set_handler(2, handler!(non_maskable_handler));
        idt.set_handler(3, isr!(breakpoint_handler));
        idt.set_handler(4, handler!(overflow_handler));
        idt.set_handler(5, handler!(bound_range_handler));
        idt.set_handler(6, handler!(invalid_opcode_handler));
        idt.set_handler(7, handler!(device_not_available_handler));
        idt.set_handler(8, handler!(double_fault_handler));
        idt.set_handler(10, handler!(invalid_tss_handler));
        idt.set_handler(11, handler!(segment_not_present_handler));
        idt.set_handler(12, handler!(stack_handler));
        idt.set_handler(13, handler!(general_protection_handler));
        idt.set_handler(14, handler!(page_fault_handler));
        idt.set_handler(16, handler!(x87_floating_point_handler));
        idt.set_handler(17, handler!(alignment_check_handler));
        idt.set_handler(18, handler!(machine_check_handler));
        idt.set_handler(19, handler!(simd_floating_point_handler));
        idt.set_handler(21, handler!(control_protection_handler));
        idt.set_handler(28, handler!(hypervisor_injection_handler));
        idt.set_handler(29, handler!(vmm_communication_handler));
        idt.set_handler(30, handler!(security_handler));
        idt
    };
}

pub fn init() {
    IDT.load();

    unsafe {
        asm!("int3", options(nomem, nostack));
    }
    log!("We made it back :)");
    divide_by_zero();
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx", out("ax") _, out("dx") _);
    }
}
