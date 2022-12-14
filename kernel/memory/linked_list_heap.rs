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

    unsafe fn remove_region(
        &mut self,
        front_pad: usize,
        back_pad: usize,
        layout: Layout,
        prev: *mut MemoryRegion,
        current: *mut MemoryRegion,
    ) -> *mut u8 {
        let c = current as *mut u8;

        if front_pad > 0 && back_pad > 0 {
            prev.write(MemoryRegion {
                next: current,
                len: (*prev).len,
            });

            let back_region = c.offset((front_pad + layout.size()) as isize) as *mut MemoryRegion;
            back_region.write(MemoryRegion {
                next: (*current).next,
                len: back_pad,
            });

            current.write(MemoryRegion {
                next: back_region,
                len: front_pad,
            });
        } else if front_pad > 0 {
            current.write(MemoryRegion {
                next: (*current).next,
                len: front_pad,
            });
        } else if back_pad > 0 {
            let back_region = c.offset((front_pad + layout.size()) as isize) as *mut MemoryRegion;
            back_region.write(MemoryRegion {
                next: (*current).next,
                len: back_pad,
            });

            prev.write(MemoryRegion {
                next: back_region,
                len: (*prev).len,
            });
        } else {
            prev.write(MemoryRegion {
                next: (*current).next,
                len: (*prev).len,
            });
        }

        c.offset(front_pad as isize)
    }

    unsafe fn fit_layout_to_region(
        layout: Layout,
        region: *mut MemoryRegion,
    ) -> Option<(usize, usize)> {
        let alignment = core::cmp::max(layout.align(), core::mem::align_of::<MemoryRegion>());
        let size = core::cmp::max(layout.size(), layout.align());

        let r = region as *mut u8;
        let addr = r.offset(r.align_offset(alignment) as isize);

        let front_pad = if addr == r {
            0
        } else {
            addr.to_bits() - r.to_bits() /* todo make min size == memoryregion? */
        };
        let back_pad = (*region).len - (layout.size() + front_pad);

        let r = region.read();
        if layout.size() > r.len {
            return None;
        }

        if back_pad > 0 && back_pad < size_of::<MemoryRegion>() {
            return None;
        }

        return Some((front_pad, back_pad));
    }

    unsafe fn find_first_fit(&mut self, layout: Layout) -> *mut u8 {
        let mut prev = null_mut();
        let mut current = self.head;

        while !current.is_null() {
            match Self::fit_layout_to_region(layout, current) {
                Some((front_padding, back_padding)) => {
                    return self.remove_region(front_padding, back_padding, layout, prev, current);
                }
                None => {
                    prev = current;
                    current = (*current).next;
                }
            }
        }

        null_mut()
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
        // TODO increase layout size if it does not meet the minimum size to match what we do in alloc()
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
