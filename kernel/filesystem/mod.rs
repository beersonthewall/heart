mod custody;
mod inode;
mod file;
mod mem_fs;
mod vfs;

use alloc::sync::Arc;
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
    /// Looks up a path in the given filesystem and returns it's Inode (if it exists).
    fn resolve_path(&self, path: &str) -> Option<Arc<RwLock<InodeTraitObject>>>;
}
