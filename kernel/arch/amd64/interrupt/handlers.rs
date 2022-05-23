use core::arch::asm;

#[derive(Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
    intruction_ptr: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

pub extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: divide by zero\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn debug_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: debug\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn non_maskable_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: non maskable interrupt\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn breakpoint_handler() {
    log!("EXCEPTION: breakpoint");
}

pub extern "C" fn overflow_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: overflow\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn bound_range_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: bound range\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: invalid opcode\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn device_not_available_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: device not available\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: double fault\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn invalid_tss_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: invalid tss\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn segment_not_present_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: segment not present\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn stack_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: stack\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn general_protection_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: general protection\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: page fault\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn x87_floating_point_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: x87 floating point\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn alignment_check_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: alignment check\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn machine_check_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: machine check\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn simd_floating_point_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: simd floating point\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn control_protection_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: control protection\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn hypervisor_injection_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: hypervisor injection\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn vmm_communication_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: vmm conmmunication\n{:?}", stack_frame);
    loop {}
}

pub extern "C" fn security_handler(stack_frame: &ExceptionStackFrame) -> ! {
    log!("EXCEPTION: security\n{:?}", stack_frame);
    loop {}
}

macro_rules! handler {
    ($name:ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("mov rdi, rsp; sub rsp, 8; call {x}; iretq",
                     x = sym $name, options(noreturn))
            }
        }
        wrapper
    }};
}

macro_rules! isr {
    ($name:ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            // sub rsp to correctly align stack ptr for extern 'C' calls
            unsafe {
                asm!("push rax
                      push rcx
                      push rdx
                      push rbx
                      push rsp
                      push rbp
                      push rsi
                      push rdi
                      push r8
                      push r9
                      push r10
                      push r11
                      mov rsi, [rsp + 9*8]
                      mov rdi, rsp
                      sub rsp, 8
                      call {x}
                      add rsp, 8
                      pop r11
                      pop r10
                      pop r9
                      pop r8
                      pop rdi
                      pop rsi
                      pop rbp
                      pop rsp
                      pop rbx
                      pop rdx
                      pop rcx
                      pop rax
                      iretq
                      ", x = sym $name, options(noreturn));
            }
        }
        wrapper
    }};
}