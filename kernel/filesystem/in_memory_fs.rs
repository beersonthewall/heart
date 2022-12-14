use super::inode::{Inode, InodeIdentifier, InodeMetadata, FileSystemIndex};
use super::FileSystem;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;


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
    inodes: BTreeMap<InodeIdentifier, Box<dyn Inode>>,
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

    fn inode(&self, inode_id: InodeIdentifier) -> Option<&Box<dyn Inode>> {
	self.inodes.get(&inode_id)
    }
}
