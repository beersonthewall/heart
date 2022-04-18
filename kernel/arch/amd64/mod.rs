// x86 port IO
#[path = "../x86_common/io.rs"]
mod x86_io;

// Debug output channel (uses serial)
#[path = "../x86_common/debug.rs"]
pub mod debug;

pub mod memory;
