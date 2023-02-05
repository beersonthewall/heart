mod custody;
mod error;
mod file;
mod inode;
mod mem_fs;
mod path;
mod vfs;

use alloc::sync::Arc;
use error::FileSystemError;
use file::FileDescriptor;
use inode::InodeTraitObject;
use mem_fs::MemFs;
use spin::rwlock::RwLock;
use vfs::VirtualFileSystem;

// Lazy static generates a type which implements Deref<RwLock<VirtualFileSystem>>
lazy_static! {
    static ref VFS: RwLock<VirtualFileSystem> = RwLock::new(VirtualFileSystem::new());
}

pub fn init() {
    let mut vfs = VFS.write();
    let in_memory_file_system = Arc::new(RwLock::new(MemFs::new()));
    vfs.mount_root(in_memory_file_system);
}

pub trait FileSystem {
    /// Looks up a path in the given filesystem and returns its Inode (if it exists).
    fn resolve_path(&self, path: &str) -> Option<Arc<RwLock<InodeTraitObject>>>;

    /// Returns the inode for the filesystem root
    fn root_inode(&self) -> Arc<RwLock<InodeTraitObject>>;
}

/// Open a file with the given flags and (optional) mode.
/// FIXME: probably need to create `mode_t` in kernel_api
/// and use that instead of `usize`.
pub fn open(path: &str, flags: isize, mode: Option<usize>) -> Result<FileDescriptor, FileSystemError> {
    let mut vfs = VFS.write();
    vfs.open(path, flags, mode)
}
