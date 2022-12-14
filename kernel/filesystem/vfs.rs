use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::__Deref;
use core::cell::RefCell;

use super::custody::Custody;
use super::error::FileSystemErr;
use super::FileSystem;
use super::path::Path;
use super::inode::{InodeIdentifier, FileSystemIndex, Inode};

pub struct VirtualFileSystem {
    root_inode: InodeIdentifier,
    filesystems: Vec<Box<dyn FileSystem>>,
    // FIXME: We don't have the concept of a 'process' yet, so things that are typically
    // per process like open file descriptors and 'working directory' etc. go here.
    open_fd: Vec<FileDescriptor>,
    working_directory: RefCell<Box<Custody>>,
}

pub struct FileDescriptor {
    fd: u64,
    inode_id: InodeIdentifier,
}

impl VirtualFileSystem {
    pub fn new() -> Self
    {
	let root_inode = InodeIdentifier::new(0, 0);
        Self {
	    root_inode,
	    filesystems: Vec::new(),
	    open_fd: Vec::new(),
	    working_directory: Custody::new(root_inode)
	}
    }

    pub fn mkdir(&mut self, path: String) {
	
    }

    pub fn open(&mut self, path: String) -> Result<FileDescriptor, FileSystemErr> {
	/*
	Steps:

	1) Parse path
	2) Follow path down from root inode, creating custodies to track parent-child relationship
	3) Create File descriptor and add it to self.open_fd
	4) return
	 */
	let custody = self.resolve_path(path);
	Ok(FileDescriptor { fd: 1, inode_id: self.root_inode })
    }

    /// Mount the provided filesystem as the root filesystem.
    pub fn mount_root(&mut self, mut filesystem: Box<dyn FileSystem>) -> Result<(), FileSystemErr>
    {
	let root_inode_id = filesystem.root_inode_id();
	// We don't run on 32bit systems so converting usize -> u64 should be okay.
	let fs_index: FileSystemIndex = self.filesystems.len() as u64;
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

    fn fetch_inode(&self, inode_id: InodeIdentifier) -> Option<&Box<dyn Inode>> {
	match self.filesystems.get(inode_id.filesystem_index() as usize) {
	    None => None,
	    Some(filesystem) => {
		filesystem.inode(inode_id)
	    }
	}
    }
}

