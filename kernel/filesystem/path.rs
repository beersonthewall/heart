/// Path iteration and convinience methods.
use alloc::vec::Vec;

pub struct Path<'a> {
    path: &'a str,
    byte_pos: usize,
}

impl<'a> Path<'a> {
    /// Create a new Path struct
    pub fn new(path: &'a str) -> Self {
	Self {
	    path,
	    byte_pos: 0,
	}
    }

    /// Is this path absolute?
    pub fn is_absolute(&self) -> bool {
	let first_slash = self.path.find('/');
	(first_slash.is_some() && first_slash.unwrap() == 0) || false
    }

    /// Returns a slice for all but the last path component.
    /// consumes self, because it mutates the iterator (i think).
    pub fn all_but_last(self) -> &'a str {
	// There has to be a better way
	let len = self.path.len();
	let path = self.path;
	let parts = self.collect::<Vec<&str>>();
	let last_part_len = parts[parts.len() - 1].len();
	&path[0..(len - last_part_len - 1)]
    }
}

impl<'a> Iterator for Path<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
	let pos = self.byte_pos;

	// Bounds check
	if pos >= self.path.len() {
	    return None;
	}

	// Increment byte_pos, rounding up to the next utf-8 code point.
	// Apparently this is only done at the character level and can
	// split graphemes?
	self.byte_pos = self.path.ceil_char_boundary(self.byte_pos + 1);

	// calculate the next path component by finding the next '/' char or
	// going to the end.
	match self.path[pos..].find('/') {
	    None => Some(&self.path[pos..]),
	    Some(index) => Some(&self.path[pos..index]),
	}
    }
}
