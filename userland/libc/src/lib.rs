#![no_std]

extern crate kernel_api;
pub mod fcntl;

mod va_list;

// Rust really cannot handle functions with variadc arguments. So we do some weird shit.
// https://www.reddit.com/r/rust/comments/2qje69/ffi_dealing_with_va_list/
// We need a pointer to a variadic argument list.
//pub type VAList = *mut core::ffi::c_void;

// C Types
pub type CChar = u8;
pub type CInt = isize; // TODO not correct
