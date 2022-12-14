mod custody;
mod error;
mod file;
mod in_memory_fs;
mod inode;
mod path;
mod vfs;

use alloc::boxed::Box;
use alloc::string::String;
use in_memory_fs::InMemoryFileSystem;
use inode::{Inode, InodeIdentifier, FileSystemIndex};
use vfs::VirtualFileSystem;

pub struct File;

pub trait FileSystem {
    fn root_inode_id(&self) -> InodeIdentifier;
    fn inode(&self, inode_id: InodeIdentifier) -> Option<&Box<dyn Inode>>;
    fn set_fs_index(&mut self, fs_index: FileSystemIndex);
}

pub fn init() {
    log!("filesystem::init()...");
    // FIXME: much like highlander, there can only be one VFS.
    let mut vfs = VirtualFileSystem::new();
    let mut in_memory_fs = InMemoryFileSystem::new(0);
    if let Err(vfs_error) = vfs.mount_root(Box::new(in_memory_fs)) {
        panic!("Failed to mount root filesystem");
    }

    vfs.resolve_path(String::from("/hello"));
    log!("filesystem::init() complete.");
}
