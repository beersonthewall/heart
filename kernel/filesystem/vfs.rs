use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::rwlock::RwLock;

use super::custody::Custody;
use super::mem_fs::MemFs;
use super::{FileSystem, FileSystemError};
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
	let custody = self.resolve_path(path, self.root_inode.clone().unwrap())?;
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

	// FIXME: This is a gnarly closure, but we don't want to break it out because we couldn't capture
	// the 'path' value. Rust explicitly delineates between closures and functions, which can avoid some
	// nasty problems (see https://users.rust-lang.org/t/inner-functions-not-closed-over-their-environment/7696/9)
	//
	// For clairty the type signature of this closure would be:
	// fn step(acc: Result<(Arc<RwLock<Custody>>, usize, usize), FileSystemError>, slash_index: &usize)
	//	-> Result<(Arc<RwLock<Custody>>, usize, usize), FileSystemError>;
	//
	// Lastly I did this via fold because it avoids liftime issues if we tried to do this with a for loop e.g.
	//
	// let mut custody = base;
	// for blah in blah {
	//   <snip>
	//   // here be issues with the borrow checker, or maybe I'm just a dummy and can't figure this out.
	//   custody = custody.lookup(name);
	// }
	//
	// FIXME: also this is forced to complete the whole iteration even if the first lookup() fails.
	// figuring out a way to get try_fold to work might allow short-circuting the evaltuation.
	let result = slash_indices.iter().fold(Ok((base, 0, 0)), |acc, slash_index| {
	    let acc = match acc {
		Ok(a) => a,
		Err(_) => return acc,
	    };

	    let custody = acc.0;
	    let count = acc.1;
	    let prev_slash_index = acc.2;
	    let slash_index = *slash_index;

	    if slash_index - prev_slash_index == 1 {
		return Ok((custody, count + 1, slash_index));
	    }

	    let name = &path[prev_slash_index + 1..slash_index];
	    let custody = match custody.read().lookup(name)? {
		Some(c) => c,
		None => return Err(FileSystemError::FileNotFound),
	    };
	    
	    Ok((custody, count + 1, slash_index))
	});
	match result {
	    Ok((custody, _, _)) => Ok(custody),
	    Err(e) => Err(e),
	}
    }

}
