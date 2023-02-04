/// Type alias so we don't forget pretty much everywhere
/// we need to use Inode trait objects we want to force Send + Sync
pub type InodeTraitObject = dyn Inode + Send + Sync;

#[derive(Debug, Clone, Copy)]
pub struct InodeIdentifier(pub usize);

pub trait Inode {
    fn metadata(&self) -> InodeMetadata;
    fn id(&self) -> InodeIdentifier;
}

#[derive(Copy, Clone)]
pub struct InodeMetadata {
    mode: u32,
}

impl InodeMetadata {
    pub fn new(mode: u32) -> Self {
	Self {
	    mode,
	}
    }
}
