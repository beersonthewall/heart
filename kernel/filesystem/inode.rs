
/// Type alias so we don't forget pretty much everywhere
/// we need to use Inode trait objects we want to force Send + Sync
pub type InodeTraitObject = dyn Inode + Send + Sync;

pub trait Inode {
    fn metadata(&self) -> InodeMetadata;
}

pub struct InodeMetadata;
