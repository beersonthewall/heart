#![allow(dead_code)]
use core::arch::asm;

/// Write a byte to the specified port
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("al") val, in("dx") port);
}

/// Read a single byte from the specified port
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    asm!("in al, dx", out("al") ret, in("dx") port);
    return ret;
}

/// Write a word (16-bits) to the specified port
pub unsafe fn outw(port: u16, val: u16) {
    asm!("outw ax, dx", in("ax") val, in("dx") port);
}

/// Read a word (16-bits) from the specified port
pub unsafe fn inw(port: u16) -> u16 {
    let ret: u16;
    asm!("inw ax, dx", out("ax") ret, in("dx") port);
    return ret;
}

/// Write a long/double-word (32-bits) to the specified port
pub unsafe fn outl(port: u16, val: u32) {
    asm!("outl eax, dx", in("eax") val, in("dx") port);
}

/// Read a long/double-word (32-bits) from the specified port
pub unsafe fn inl(port: u16) -> u32 {
    let ret: u32;
    asm!("inl dx, eax", out("eax") ret, in("dx") port);
    return ret;
}
