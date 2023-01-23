use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use core::cell::RefCell;

use super::custody::Custody;
use super::error::FileSystemErr;
use super::FileSystem;
use super::path::Path;
use super::inode::{InodeIdentifier, FileSystemIndex, Inode};
use super::file::{FileDescriptor, OpenFileDescription};
use kernel_api::headers::fcntl::{
    O_APPEND,
    O_CREAT,
    O_RDONLY,
    O_RDWR,
    O_WRONLY,
};

pub struct VirtualFileSystem {
    root_inode: InodeIdentifier,
    filesystems: Vec<Box<dyn FileSystem>>,
    // FIXME: We don't have the concept of a 'process' yet, so things that are typically
    // per process like open file descriptors and 'working directory' etc. go here.
    open_fd: BTreeMap<FileDescriptor, OpenFileDescription>,
    working_directory: RefCell<Box<Custody>>,
    next_fd: usize,
}

impl VirtualFileSystem {
    pub fn new() -> Self
    {
	let root_inode = InodeIdentifier::new(0, 0);
        Self {
	    root_inode,
	    filesystems: Vec::new(),
	    open_fd: BTreeMap::new(),
	    working_directory: Custody::new(root_inode),
	    next_fd: 0,
	}
    }

    pub fn mkdir(&mut self, path: String) {
	
    }

    pub fn read(&mut self) {

    }

    /// [write() spec link](https://pubs.opengroup.org/onlinepubs/9699919799/functions/write.html)
    pub fn write(&mut self, fd: FileDescriptor, bytes: &[u8]) -> Result<(), FileSystemErr> {
	let open_file_description = match self.open_fd.get_mut(&fd) {
	    None => return Err(FileSystemErr::EBadF),
	    Some(metadata) => metadata,
	};

	// If the file is not open for writing return EBADF
	if open_file_description.has_flags(O_RDONLY) || !open_file_description.has_flags(O_RDWR | O_WRONLY) {
	    return Err(FileSystemErr::EBadF)
	}

	if open_file_description.has_flags(kernel_api::headers::fcntl::O_APPEND) {
	    // Can't call fetch_inode() because it takes an immutable ref and we can have a &mut self and &self
	    // at the same time.
	    let file_size = match self.filesystems.get(fd.inode_id().filesystem_index() as usize) {
		None => return Err(FileSystemErr::FileNotFound),
		Some(filesystem) => {
		    match filesystem.inode(fd.inode_id()) {
			None => return Err(FileSystemErr::FileNotFound),
			Some(inode) => inode.metadata().size(),
		    }
		}
	    };
	    // O_APPEND moves cursor to the end.
	    open_file_description.set_offset(file_size - 1);
	}

	match self.filesystems.get_mut(fd.inode_id().filesystem_index()) {
	    None => Err(FileSystemErr::FileNotFound),
	    Some(fs) => fs.write(fd, bytes, open_file_description),
	}
    }

    pub fn open(&mut self, path: String, options: u32) -> Result<FileDescriptor, FileSystemErr> {
	let custody = self.resolve_path(path)?;

	// Check if we need to create the file.
	if options & O_CREAT != 0 {
	    self.create(path, options)?;
	}
	
	// Need to store inode id in tmp variable for compiler to figure out
	// that it can copy the resulting inode id and not care that
	// the custody gets dropped right after we return.
	let inode_id = custody.borrow().as_ref().inode();
	let fd = FileDescriptor::new(self.next_fd, inode_id);
	self.open_fd.insert(fd, OpenFileDescription::new());
	Ok(fd)
    }

    /// Mount the provided filesystem as the root filesystem.
    pub fn mount_root(&mut self, mut filesystem: Box<dyn FileSystem>) -> Result<(), FileSystemErr>
    {
	let root_inode_id = filesystem.root_inode_id();
	let fs_index: FileSystemIndex = self.filesystems.len();
	filesystem.set_fs_index(fs_index);
	self.root_inode = root_inode_id;
	self.filesystems.push(filesystem);
	Ok(())
    }

    pub fn resolve_path(&self, path: String) -> Result<RefCell<Box<Custody>>, FileSystemErr> {
	let path = Path::new(path);
	let mut path_iterator = path.components();

	let first = path_iterator.next();
	if let None = first {
	    return Err(FileSystemErr::FileNotFound);
	}

	// Set the 'base' custody.
	// Custodies are a linked list used to answer the question:
	// "What's the parent of a given inode?"
	let is_absolute = first.unwrap() == "/";
	let mut base = if is_absolute {
	    Custody::new(self.root_inode)
	} else {
	    self.working_directory.clone()
	};

	let mut parent = base;
	// Build up the chain of custodies checking for errors as we go.
	for part in path_iterator {
	    let parent_inode = match self.fetch_inode((*parent.borrow()).inode()) {
		None => return Err(FileSystemErr::FileNotFound),
		Some(inode) if !inode.metadata().is_directory() => {
		    return Err(FileSystemErr::ENotDir);
		},
		Some(inode) => {
		    inode
		},
	    };

	    let child_inode_id = match parent_inode.find_child(part) {
		None => return Err(FileSystemErr::FileNotFound),
		Some(id) => id,
	    };

	    let child_custody = Custody::new(child_inode_id);
	    parent = child_custody;
	}

	Ok(parent)
    }

    fn create(&mut self, path: &str, options: u32) -> Result<(), FileSystemErr> {
	assert!(options & O_CREAT != 0);
	Ok(())
    }

    fn fetch_inode(&self, inode_id: InodeIdentifier) -> Option<Rc<Box<dyn Inode>>> {
	match self.filesystems.get(inode_id.filesystem_index() as usize) {
	    None => None,
	    Some(filesystem) => {
		filesystem.inode(inode_id)
	    }
	}
    }
}

