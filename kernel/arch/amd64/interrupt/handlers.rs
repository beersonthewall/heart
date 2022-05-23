use core::arch::asm;

pub extern "C" fn divide_by_zero_handler() -> ! {
    log!("EXCEPTION: divide by zero");
    loop {}
}

pub extern "C" fn breakpoint_handler() {
    log!("EXCEPTION: breakpoint");
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
