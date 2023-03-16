use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use kernel_api::headers::fcntl::{
    O_CREAT,
};
use spin::rwlock::RwLock;

use super::{FileSystem, FileSystemError};
use super::custody::Custody;
use super::mem_fs::MemFs;
use super::path::Path;
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
    root_inode: Option<Arc<RwLock<Custody>>>,
}

impl VirtualFileSystem {

    /// Create a new virtual file system with *no* root inode reference.
    /// No initialization happens here. That will be done in [].
    pub fn new() -> Self {
	Self {
	    file_systems: RwLock::new(Vec::new()),
	    open_files: BTreeMap::new(),
	    custody_cache: Vec::new(),
	    root_inode: None,
	}
    }

    /// Mounts the provided file system to "/"
    pub fn mount_root(&mut self, root_file_system: Arc<RwLock<dyn FileSystem + Send + Sync>>) {
	self.file_systems.write().push(root_file_system.clone());

	let name = String::from("/");
	let parent = None;

	// Acquire read lock on root fs
	let root_fs = root_file_system.read();

	let inode = match root_fs.resolve_path("/") {
	    None => root_fs.root_inode(),
	    Some(inode) => inode,
	};

	// FIXME: implement inode.metadata().is_direcotry() and check

	let root = Arc::new(RwLock::new(Custody::new(name, parent, inode)));
	self.custody_cache.push(root);
    }

    /// Opens a file.
    /// Spec link: https://pubs.opengroup.org/onlinepubs/9699919799/
    pub fn open(&self, path: &str, flags: isize, mode: Option<usize>) -> Result<FileDescriptor, FileSystemError> {
	// TODO: likely checked at the LIBC layer in the future? but doesn't hurt to check here.
	if path.len() == 0 {
	    return Err(FileSystemError::EFAULT);
	}

	let o_creat = flags & (O_CREAT as isize) > 0;
	let mut new_fd: Option<FileDescriptor> = None;
	if o_creat {
	    let lookup_path = Path::new(path).all_but_last();
	    let custody = self.resolve_path(lookup_path, self.root_inode.clone().unwrap())?;
	    let name_to_create = Path::new(path).last();
	    // Pick a default mode, not sure this is correct :/
	    let mode = if let Some(m) = mode { m } else { 0x0644 };
	    new_fd = Some(custody.add_child(name_to_create, mode)?);
	}

	

	let root_inode = self.root_inode.clone().unwrap();
	let custody = self.resolve_path(path, root_inode)?;
	Ok(FileDescriptor(1))
    }

    /// Resolve path name and return a custody which links the name to an Inode.
    /// Can provide None as the base custody which will default to the root inode.
    fn resolve_path(&self, path: &str, base: Arc<RwLock<Custody>>) -> Result<Arc<RwLock<Custody>>, FileSystemError> {
	let chars = path.char_indices();
	let mut slash_indices = Vec::new();

	chars.for_each(|(i, c)| {
	    if c == '/' {
		slash_indices.push(i);
	    }
	});

	let mut current_custody = base;
	let mut name_start = 0;
	for index in slash_indices {
	    if index == name_start && index == 0 {
		continue;
	    }

	    if index - name_start == 1 {
		name_start = index;
		continue;
	    }

	    let child = match current_custody.read().lookup(&path[name_start..index])? {
		None => return Err(FileSystemError::FileNotFound),
		Some(custody) => custody,
	    };
	    current_custody = child;
	}
	Ok(current_custody)
    }

}
