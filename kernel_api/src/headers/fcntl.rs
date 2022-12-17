// Constants
pub const O_RDONLY: u32 = 1 << 0;
pub const O_WRONLY: u32 = 1 << 1;
pub const O_RDWR: u32 = O_RDONLY | O_WRONLY;
pub const O_ACCMODE: u32 = O_RDONLY | O_WRONLY;
pub const O_EXEC: u32 = 1 << 2;
pub const O_CREAT: u32 = 1 << 3;
pub const O_EXCL: u32 = 1 << 4;
pub const O_NOCTTY: u32 = 1 << 5;
pub const O_TRUNC: u32 = 1 << 6;
pub const O_APPEND: u32 = 1 << 7;
pub const O_NONBLOCK: u32 = 1 << 8;
pub const O_DIRECTORY: u32 = 1 << 9;
pub const O_NOFOLLOW: u32 = 1 << 10;
pub const O_CLOEXEC: u32 = 1 << 11;
pub const O_DIRECT: u32 = 1 << 12;
pub const O_SYNC: u32 = 1 << 13;
