use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::rwlock::RwLock;

use super::inode::InodeTraitObject;

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
}
