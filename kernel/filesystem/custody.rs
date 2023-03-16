use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::rwlock::RwLock;

use super::file::FileDescriptor;
use super::inode::InodeTraitObject;
use super::error::FileSystemError;
/// A Custody maps a human readable name to the more computer friendly
/// Inodes. Similar to linux dentry (the name of this struct is borrowed from
/// Serenity OS).
pub struct Custody {
    // FIXME: find a way to statically allocate smaller names
    // for example linux dentry does `unsigned char d_iname[DNAME_INLINE_LEN];`
    name: String,
    // Only one custody can have no parent, "/"
    // FIXME: perhaps this should be better encoded in the type system?
    parent: Option<Arc<RwLock<Custody>>>,
    inode: Arc<RwLock<InodeTraitObject>>,
    children: Vec<Arc<RwLock<Custody>>>,
}

impl Custody {
    pub fn new(name: String, parent: Option<Arc<RwLock<Custody>>>, inode: Arc<RwLock<InodeTraitObject>>) -> Self {
	Self {
	    name,
	    parent,
	    inode,
	    children: Vec::new(),
	}
    }

    /// Adds a child to the inode associated with this Custody.
    pub fn add_child(&self, name: &str, mode: usize) -> Result<FileDescriptor, FileSystemError> {
	
	Ok(FileDescriptor(1))
    }

    pub fn name(&self) -> &str {
	&self.name
    }

    pub fn lookup(&self, name: &str) -> Result<Option<Arc<RwLock<Custody>>>, FileSystemError> {
	for c in &self.children {
	    if name == c.read().name() {
		return Ok(Some(c.clone()));
	    }
	}
	Ok(None)
    }
}
