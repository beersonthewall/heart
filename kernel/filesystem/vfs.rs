use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::rwlock::RwLock;

use super::custody::Custody;
use super::mem_fs::MemFs;
use super::FileSystem;
use super::file::{
    File, FileDescriptor
};

/// A type alias to make the type sig a little less gross
type FileSystemList = RwLock<Vec<Arc<RwLock<dyn FileSystem + Send + Sync>>>>;

/// Virtual Filesystem Implementation
///
/// Keeps a list of open files, filesystems, and the custody cache. Custodies
/// are akin to dentry in linux. They map between human readable
/// names and computer-friendly names for inodes.
pub struct VirtualFileSystem {
    file_systems: FileSystemList,
    open_files: BTreeMap<FileDescriptor, Arc<RwLock<File>>>,
    custody_cache: Vec<Arc<RwLock<Custody>>>,
    root: Option<Arc<RwLock<Custody>>>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
	Self {
	    file_systems: RwLock::new(Vec::new()),
	    open_files: BTreeMap::new(),
	    custody_cache: Vec::new(),
	    root: None,
	}
    }

    /// Mounts the provided file system to "/"
    pub fn mount_root(&mut self, root_file_system: Arc<RwLock<dyn FileSystem + Send + Sync>>) {
	self.file_systems.write().push(root_file_system.clone());

	let name = String::from("/");
	let parent = None;

	let inode = match root_file_system.read().resolve_path("/") {
	    None => { panic!(); },
	    Some(inode) => inode,
	};

	let root = Arc::new(RwLock::new(Custody::new(name, parent, inode)));
	self.custody_cache.push(root);
    }
}
