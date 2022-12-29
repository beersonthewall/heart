use super::inode::InodeIdentifier;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct FileDescriptor {
    fd: usize,
    inode_id: InodeIdentifier,
}

impl FileDescriptor {
    pub fn new(fd: usize, inode_id: InodeIdentifier) -> Self {
	Self {
	    fd,
	    inode_id,
	}
    }

    pub fn inode_id(&self) -> InodeIdentifier {
	self.inode_id
    }
}

pub struct OpenFileDescription {
    flags: u32,
    offset: usize,
}

impl OpenFileDescription {
    pub fn new() -> Self {
	Self {
	    flags: 0,
	    offset: 0,
	}
    }

    pub fn offset(&self) -> usize { self.offset }

    pub fn set_offset(&mut self, offset: usize) {
	self.offset = offset;
    }

    pub fn has_flags(&self, flags: u32) -> bool  {
	(self.flags & flags) != 0
    }
}
