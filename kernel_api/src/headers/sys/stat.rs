pub const S_IFMT: u32 = 0170000;
pub const S_IFDIR: u32 = 0040000;
pub const S_IFCHR: u32 = 0020000;
pub const S_IFBLK: u32 = 0060000;
pub const S_IFREG: u32 = 0100000;
pub const S_IFIFO: u32 = 0010000;
pub const S_IFLNK: u32 = 0120000;
pub const S_IFSOCK: u32 = 0140000;

pub const S_ISUID: u32 = 04000;
pub const S_ISGID: u32 = 02000;
pub const S_ISVTX: u32 = 01000;
pub const S_IRUSR: u32 = 0400;
pub const S_IWUSR: u32 = 0200;
pub const S_IXUSR: u32 = 0100;
pub const S_IREAD: u32 = S_IRUSR;
pub const S_IWRITE: u32 = S_IWUSR;
pub const S_IEXEC: u32 = S_IXUSR;
pub const S_IRGRP: u32 = 0040;
pub const S_IWGRP: u32 = 0020;
pub const S_IXGRP: u32 = 0010;
pub const S_IROTH: u32 = 0004;
pub const S_IWOTH: u32 = 0002;
pub const S_IXOTH: u32 = 0001;

