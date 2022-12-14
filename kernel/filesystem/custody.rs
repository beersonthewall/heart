use alloc::boxed::Box;
use core::cell::RefCell;

use super::inode::InodeIdentifier;

#[derive(Clone)]
pub struct Custody {
    parent: Option<RefCell<Box<Custody>>>,
    inode_id: InodeIdentifier,
}

impl Custody {
    pub fn new(inode_id: InodeIdentifier) -> RefCell<Box<Self>> {
	RefCell::new(
	    Box::new(Self {
	    parent: None,
	    inode_id,
	}))
    }

    pub fn append(custody: RefCell<Box<Custody>>, inode_id: InodeIdentifier) -> Custody {
	Custody {
	    parent: Some(custody),
	    inode_id,
	}
    }

    pub fn inode(&self) -> InodeIdentifier {
	self.inode_id
    }
}
