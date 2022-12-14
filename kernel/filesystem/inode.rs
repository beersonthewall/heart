use super::error::FileSystemErr;

pub trait Inode {
    fn metadata(&self) -> &InodeMetadata;
    fn read_inode(&self) -> Result<(), FileSystemErr>;
    fn find_child(&self, name: &str) -> Option<InodeIdentifier>;
    /*
    Future API, but not yet ready to implement:
    fn write_inode(&mut self) -> Result<(), FileSystemErr>;
    */
}

pub type InodeIndex = u64;
pub type FileSystemIndex = u64;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct InodeIdentifier {
    filesystem_id: FileSystemIndex,
    inode_id: InodeIndex,
}

impl InodeIdentifier {
    pub fn new(filesystem_id: FileSystemIndex, inode_id: InodeIndex) -> Self {
	Self {
	    filesystem_id,
	    inode_id,
	}
    }

    pub fn filesystem_index(&self) -> FileSystemIndex {
	self.filesystem_id
    }
}

pub struct InodeMetadata {
    mode: u16,
}

// From POSIX stat.h
const S_IFMT: u16 = 0o170000;
const S_IFDIR: u16 = 0o40000;

impl InodeMetadata {
    pub fn is_directory(&self) -> bool {
	self.mode & S_IFMT == S_IFDIR
    }
}
