use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::rwlock::RwLock;

use super::{
    FileSystem,
    error::FileSystemError,
};
use super::file::FileDescriptor;
use super::inode::{InodeTraitObject, Inode, InodeMetadata, InodeIdentifier};

/// Inode implementation for the in memory file system
struct MemFile {
    id: InodeIdentifier,
    cursor: usize,
    metadata: InodeMetadata,
    data: Arc<RwLock<Vec<u8>>>,
    children: Vec<Arc<RwLock<MemFile>>>,
}

impl MemFile {
    fn new() -> Self {
	Self {
	    id: InodeIdentifier(0),
	    cursor: 0,
	    // FIXME: don't default mode to zero
	    metadata: InodeMetadata::new(0),
	    data: Arc::new(RwLock::new(Vec::new())),
	    children: Vec::new(),
	}
    }
}

unsafe impl Send for MemFile {}
unsafe impl Sync for MemFile {}

impl Inode for MemFile {
    fn metadata(&self) -> InodeMetadata {
	self.metadata
    }

    fn id(&self) -> InodeIdentifier {
	self.id
    }

    fn add_child(&self, mode: usize) -> Result<FileDescriptor, FileSystemError> {
	Ok(FileDescriptor(1))
    }
}

pub struct MemFs {
    root_inode: Arc<RwLock<MemFile>>,
}

impl MemFs {
    pub fn new() -> Self {
	Self {
	    root_inode: Arc::new(RwLock::new(MemFile::new())),
	}
    }
}

unsafe impl Send for MemFs {}
unsafe impl Sync for MemFs {}

impl FileSystem for MemFs {
    fn resolve_path(&self, path: &str) -> Option<Arc<RwLock<InodeTraitObject>>> {
	None
    }

    fn root_inode(&self) -> Arc<RwLock<InodeTraitObject>> {
	self.root_inode.clone()
    }
}
