use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::collections::{BTreeMap, TryReserveError};
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

use super::file::{FileDescriptor, OpenFileDescription};
use super::inode::{Inode, InodeIdentifier, InodeMetadata, FileSystemIndex};
use super::FileSystem;
use super::error::FileSystemErr;
use super::custody::Custody;
use super::path::Path;

const INITIAL_SIZE: usize = 2048;

struct MemFile {
    bytes: Vec<u8>,
    parent: u64,
    children: BTreeMap<String, InodeIdentifier>,
    name: String,
    metadata: InodeMetadata,
}

impl Inode for MemFile {
    fn metadata(&self) -> &InodeMetadata {
	&self.metadata
    }

    fn write(&mut self, bytes: &[u8], open_file_description: &OpenFileDescription) -> Result<(), FileSystemErr> {
	let start = open_file_description.offset();
	let end = bytes.len() + 1;
	// If we're overwriting already alloc'd space we're okay to just do that.
	if end < self.bytes.len() {
	    self.bytes.as_mut_slice()[start..end].copy_from_slice(bytes);
	}

	// Check that we can allocate enough space
	let size_diff = end - self.bytes.len();
	match self.bytes.try_reserve(size_diff) {
	    Ok(()) => self.bytes.as_mut_slice()[start..end].copy_from_slice(bytes),
	    Err(TryReserveError) => return Err(FileSystemErr::ENOMEM),
	}

	Ok(())
    }

    fn read_inode(&self) -> Result<(), super::error::FileSystemErr> {
        todo!()
    }

    fn find_child(&self, name: &str) -> Option<InodeIdentifier> {
	match self.children.get(name) {
	    Some(&id) => Some(id),
	    None => None,
	}
    }
}

pub struct InMemoryFileSystem {
    files: BTreeMap<InodeIdentifier, MemFile>,
    inodes: BTreeMap<InodeIdentifier, Rc<Box<dyn Inode>>>,
    next_inode: InodeIdentifier,
    fs_index: FileSystemIndex,
}

impl InMemoryFileSystem {
    pub fn new(fs_index: FileSystemIndex) -> Self {
        Self {
            files: BTreeMap::new(),
            inodes: BTreeMap::new(),
            next_inode: InodeIdentifier::new(fs_index, 0),
	    fs_index,
        }
    }
}

impl FileSystem for InMemoryFileSystem {
    fn set_fs_index(&mut self, fs_index: FileSystemIndex) {
	self.fs_index = fs_index;
    }

    fn root_inode_id(&self) -> InodeIdentifier {
	self.next_inode
    }

    fn inode(&self, inode_id: InodeIdentifier) -> Option<Rc<Box<dyn Inode>>> {
	self.inodes.get(&inode_id).cloned()
    }

    fn write(&mut self, fd: FileDescriptor, bytes: &[u8], open_file_description: &OpenFileDescription) -> Result<(), FileSystemErr> {
	match self.files.get_mut(&fd.inode_id()) {
	    None => Err(FileSystemErr::FileNotFound),
	    Some(file) => file.write(bytes, open_file_description),
	}
    }

    fn make_inode(&mut self, custody: RefCell<Box<Custody>>, path: String, mode: u32) -> Result<(), FileSystemErr> {
	Ok(())
    }
}
