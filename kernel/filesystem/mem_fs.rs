use alloc::sync::Arc;
use spin::rwlock::RwLock;

use super::FileSystem;
use super::inode::InodeTraitObject;

pub struct MemFs {}

impl MemFs {
    pub fn new() -> Self {
	Self {}
    }
}

unsafe impl Send for MemFs {}
unsafe impl Sync for MemFs {}

impl FileSystem for MemFs {
    fn resolve_path(&self, path: &str) -> Option<Arc<RwLock<InodeTraitObject>>> {
	None
    }
}
