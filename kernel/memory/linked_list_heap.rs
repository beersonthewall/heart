use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use spin::mutex::Mutex;

pub struct LinkedListHeap {
    inner: Mutex<LinkedListHeapInner>,
}

impl LinkedListHeap {
    pub fn new(start: *mut u8, len: usize) -> Self {
        Self {
            inner: Mutex::new(LinkedListHeapInner::new(start, len)),
        }
    }
}

unsafe impl GlobalAlloc for LinkedListHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let ref mut ll_heap_inner = *self.inner.lock() {
            ll_heap_inner.alloc(layout)
        } else {
            panic!("Linked list heap lock poisoned")
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let ref mut ll_heap_inner = *self.inner.lock() {
            ll_heap_inner.dealloc(ptr, layout);
        } else {
            panic!("Linked list heap lock poisoned");
        }
    }
}

struct LinkedListHeapInner {
    head: *mut MemoryRegion,
}

#[repr(C)]
struct MemoryRegion {
    next: *mut MemoryRegion,
    len: usize,
}

impl LinkedListHeapInner {
    fn new(start: *mut u8, len: usize) -> Self {
        let head = start as *mut MemoryRegion;
        unsafe {
            (*head).next = core::ptr::null_mut();
            (*head).len = len;
        }
        Self { head }
    }

    fn pop(&mut self) -> *mut MemoryRegion {
        let current = self.head;
        unsafe {
            self.head = (*self.head).next;
        }
        current
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
	if layout.size() == 0 {
	    return core::ptr::null_mut();
	}

        let mut total = 0;
        let mut count = 0;
        let mut current = self.head;

        while total < layout.size() && !current.is_null() {
            count += 1;
            total += (*current).len;
            current = (*self.head).next;
        }

        if current.is_null() && total < layout.size() {
            // OOM, maybe just panic instead?
            return core::ptr::null_mut();
        }

        // Hold on to start of the allocation to return after we're done
        let head = self.head;

        // Remove node(s) from the heap
	(0..count).for_each(|_| { self.pop(); });

        // Lastly we need to check if there's leftover space in the last node that
        // should be added to the new head.
        let difference = total - layout.size();

        // If not large enough, we can just move on with our lives.
        // Can possibly detect when merging in dealloc() and reclaim because these will
        // be holes smaller than the max slab size which shouldn't happen any
        // other way besides this since we don't use this allocator as a stand-alone
        // but rather a backup to the slab allocator.
        if difference > core::mem::size_of::<MemoryRegion>() {
            let ptr = current as *mut u8;
            let ptr = ptr.offset(difference as isize);
            let ptr = ptr as *mut MemoryRegion;
            (*ptr).next = self.head;
            (*ptr).len = difference;
            self.head = ptr;
        }

        return head as *mut u8;
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if self.head.is_null() {
            let ptr = ptr as *mut MemoryRegion;
            let node = &mut *ptr;
            node.next = core::ptr::null_mut();
            node.len = layout.size();
            return;
        }

        let mut current = self.head;
        let ptr = ptr as *mut MemoryRegion;

        while ptr > current && !current.is_null() {
            current = (*current).next;
        }

        assert!(ptr != current);

        // TODO check if the chunk we're dealloc()'ing is mergeable with either
        // the node before or the node after (AKA when combined they make a continuous address range)
        // for example { ptr: 0x100, len: 10} and {ptr: 0x110, len: 5} can just become { ptr: 0x100, len: 15 }

        (*ptr).len = layout.size();
        (*ptr).next = (*current).next;
        (*current).next = ptr;
    }

    #[allow(dead_code)]
    unsafe fn debug_heap(&mut self) {
        if self.head.is_null() {
            log!("Linked List Allocator is empty");
            return;
        }

        let mut tmp = self.head;
        while !tmp.is_null() {
            log!("addr: {:?}, len: {}", tmp, (*tmp).len);
            tmp = (*tmp).next;
        }
    }
}
