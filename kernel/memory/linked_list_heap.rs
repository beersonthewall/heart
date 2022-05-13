use alloc::alloc::{GlobalAlloc, Layout};
use core::mem::{align_of, size_of};
use core::ptr::null_mut;
use spin::mutex::Mutex;

pub struct LinkedListHeap {
    inner: Mutex<LinkedListHeapInner>,
}

impl LinkedListHeap {
    pub unsafe fn new(start: *mut u8, len: usize) -> Self {
        Self {
            inner: Mutex::new(LinkedListHeapInner::new(start, len)),
        }
    }
}

unsafe impl GlobalAlloc for LinkedListHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ref mut ll_heap_inner = *self.inner.lock();
        ll_heap_inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ref mut ll_heap_inner = *self.inner.lock();
        ll_heap_inner.dealloc(ptr, layout)
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
    unsafe fn new(start: *mut u8, len: usize) -> Self {
        let head = start.cast::<MemoryRegion>();
        let head = head.add(align_of::<MemoryRegion>());
        head.write(MemoryRegion {
            next: null_mut(),
            len,
        });
        Self { head }
    }

    unsafe fn find_first_fit(&mut self, layout: Layout) -> *mut u8 {
        self.debug_heap();

        let mut prev_ptr = null_mut();
        let mut cur_ptr = self.head;

        loop {

            if cur_ptr.is_null() {
                return null_mut();
            }
            let current = cur_ptr.read();

            let raw_ptr = cur_ptr as *mut u8;
            let front_pad = if raw_ptr.align_offset(layout.align()) < size_of::<MemoryRegion>() {
                raw_ptr.align_offset(layout.align()) + layout.align()
            } else {
                raw_ptr.align_offset(layout.align())
            };

            let layout_size = front_pad + layout.size();
            let back_pad = current.len - layout_size;

            log!("fp: {}, lsz: {}, laln: {}, bp: {}, o: {}", front_pad, layout_size, layout.align(), back_pad, raw_ptr.align_offset(layout.align()));
            if layout_size > current.len || (back_pad > 0 && back_pad < size_of::<MemoryRegion>()) {
                prev_ptr = cur_ptr;
                cur_ptr = current.next;
                continue;
            }

            let next_region = if back_pad > 0 {
                let back_region = cur_ptr.add(layout_size);
                back_region.write(MemoryRegion {
                    next: current.next,
                    len: back_pad,
                });

                back_region
            } else {
                current.next
            };

            if front_pad > 0 {
                cur_ptr.write(MemoryRegion {
                    next: next_region,
                    len: front_pad,
                });
            } else if prev_ptr.is_null() {
                self.head = next_region;
            } else {
                (*prev_ptr).next = next_region;
            }

            // [p1, p2] [c1, c2] [n1, n2]
            // null [c1, c2] null
            // null [c1, c2] [n1, n2]
            // [p1, p2] [c1, c2] null

            // TODO Merge

            self.debug_heap();
            return cur_ptr.add(front_pad) as *mut u8;
        }
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return null_mut();
        }

        let mut sz = layout.size();
        if sz < size_of::<MemoryRegion>() {
            sz = size_of::<MemoryRegion>();
        }
        let layout = Layout::from_size_align(sz, layout.align())
            .expect("Failure to create layout with a minimum size.");

        self.find_first_fit(layout)
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
        log!("debugging heap...");
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
