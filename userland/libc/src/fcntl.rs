use crate::{CChar, CInt};
use super::va_list::VAList;
use kernel_api::prelude::*;
use kernel_api::headers::{
    errno
};

// Re-exports
pub use kernel_api::headers::fcntl;

#[no_mangle]
pub extern "C" fn open(path: *const CChar, flags: isize, va_list: VAList) -> CInt {
    if path.is_null() {
	return errno::ErrnoCode::EFAULT as isize;
    }

    // FIXME: actually implement VAList and unpack the last param (mode_t mode) if it exists

    let fd = match fcntl::open(path, flags, None) {
	Ok(fd) => fd,
	Err(e) => return -1,
    };

    0
}

#[cfg(test)]
mod tests {
    use crate::va_list::VAList;
    use super::*;
    use kernel_api::headers::errno;

    #[test]
    fn invalid_path() {
	let result = open(core::ptr::null(), 0, VAList {});
	assert_eq!(errno::ErrnoCode::EINVAL as isize, result);
    }
}
