use alloc::string::String;
use core::str::CharIndices;
use core::iter::{IntoIterator, Peekable};

pub struct Path<'a> {
    path: String,
    _spooky: core::marker::PhantomData<&'a str>
}

impl<'a> Path<'a> {
    pub fn new(path: String) -> Self {
	Self {
	    path,
	    _spooky: core::marker::PhantomData,
	}
    }

    pub fn components(&self) -> PathIter {
	PathIter::new(self.path.as_ref())
    }
}

pub struct PathIter<'a> {
    path: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> PathIter<'a> {
    fn new(path: &'a str) -> Self {
	Self {
	    path,
	    chars: path.char_indices().peekable(),
	}
    }
}

impl<'a> Iterator for PathIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
	match self.chars.next() {
	    None => None,
	    Some((index, next)) => {
		let start_index = index;
		if index == 0 && next == '/' {
		    return Some(&self.path[0..index])
		}

		while let Some((index, next)) = self.chars.peek() {
		    if *next == '/' {
			return Some(&self.path[start_index..*index]);
		    }

		    self.chars.next();
		}

		return Some(&self.path[start_index..]);
	    }
	}
    }
}
