use alloc::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

use super::addr::VirtualAddress;
use super::linked_list_heap::LinkedListHeap;

const INITIAL_HEAP_SIZE: usize = 2 * 1024 * 1024;

#[global_allocator]
static mut HEAP: Heap = Heap::new();

pub fn init(heap_start: usize) {
    unsafe {
        let aligned_heap_start_ptr = heap_start as *mut u8;
        let aligned_heap_start_ptr = aligned_heap_start_ptr
            .add(aligned_heap_start_ptr.align_offset(core::mem::size_of::<*mut u8>()));
        let heap_start = VirtualAddress::new(aligned_heap_start_ptr.to_bits());
        let _ = crate::arch::memory::map(heap_start, INITIAL_HEAP_SIZE).unwrap();
        let heap = HeapInner::new(heap_start.0 as *mut u8);
        HEAP.inner = Mutex::new(Some(heap));
    }
}

enum SlabSize {
    Slab16,
    Slab32,
    Slab64,
    Slab128,
    Slab256,
    Slab512,
}

impl SlabSize {
    const fn maximum() -> usize {
        512
    }

    fn len(&self) -> usize {
        match self {
            Slab16 => 16,
            Slab32 => 32,
            Slab64 => 64,
            Slab128 => 128,
            Slab256 => 256,
            Slab512 => 512,
        }
    }

    fn pick_slab_size(size: usize) -> Option<SlabSize> {
        if size <= 16 {
            Some(SlabSize::Slab16)
        } else if size <= 32 {
            Some(SlabSize::Slab32)
        } else if size <= 64 {
            Some(SlabSize::Slab64)
        } else if size <= 128 {
            Some(SlabSize::Slab128)
        } else if size <= 256 {
            Some(SlabSize::Slab256)
        } else if size <= 512 {
            Some(SlabSize::Slab512)
        } else {
            None
        }
    }
}

struct Heap {
    inner: Mutex<Option<HeapInner>>,
}

impl Heap {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

unsafe impl Sync for Heap {}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(ref mut heap_inner) = *self.inner.lock() {
            heap_inner.alloc(layout)
        } else {
            panic!("Global allocation error: unable to acquire heap lock for alloc()")
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ref mut heap_inner) = *self.inner.lock() {
            heap_inner.dealloc(ptr, layout);
        } else {
            panic!("Global allocation error: unable to acquire heap lock for dealloc().")
        }
    }
}

struct HeapInner {
    slab_16_bytes: Slab,
    slab_32_bytes: Slab,
    slab_64_bytes: Slab,
    slab_128_bytes: Slab,
    slab_256_bytes: Slab,
    slab_512_bytes: Slab,
    linked_list_allocator: LinkedListHeap,
}

impl HeapInner {
    unsafe fn new(heap_start: *mut u8) -> Self {
        let allocation_size = INITIAL_HEAP_SIZE / 7;
        Self {
            slab_16_bytes: Slab::new(
                heap_start.offset(0 * allocation_size as isize),
                allocation_size,
                SlabSize::Slab16,
            ),
            slab_32_bytes: Slab::new(
                heap_start.offset(1 * allocation_size as isize),
                allocation_size,
                SlabSize::Slab32,
            ),
            slab_64_bytes: Slab::new(
                heap_start.offset(2 * allocation_size as isize),
                allocation_size,
                SlabSize::Slab64,
            ),
            slab_128_bytes: Slab::new(
                heap_start.offset(3 * allocation_size as isize),
                allocation_size,
                SlabSize::Slab128,
            ),
            slab_256_bytes: Slab::new(
                heap_start.offset(4 * allocation_size as isize),
                allocation_size,
                SlabSize::Slab256,
            ),
            slab_512_bytes: Slab::new(
                heap_start.offset(5 * allocation_size as isize),
                allocation_size,
                SlabSize::Slab512,
            ),
            linked_list_allocator: LinkedListHeap::new(
                heap_start.offset(6 * allocation_size as isize),
                allocation_size,
            ),
        }
    }

    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if layout.size() > SlabSize::maximum() {
            return unsafe { self.linked_list_allocator.alloc(layout) };
        }

        let slab = match SlabSize::pick_slab_size(layout.size()) {
            Some(SlabSize::Slab16) => &mut self.slab_16_bytes,
            Some(SlabSize::Slab32) => &mut self.slab_32_bytes,
            Some(SlabSize::Slab64) => &mut self.slab_64_bytes,
            Some(SlabSize::Slab128) => &mut self.slab_128_bytes,
            Some(SlabSize::Slab256) => &mut self.slab_256_bytes,
            Some(SlabSize::Slab512) => &mut self.slab_512_bytes,
            None => return core::ptr::null_mut(),
        };

        if let Some(ptr) = slab.free_list.pop() {
            return ptr as *mut u8;
        }

        core::ptr::null_mut()
    }

    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if layout.size() > SlabSize::maximum() {
            unsafe {
                self.linked_list_allocator.dealloc(ptr, layout);
            }
            return;
        }

        let slab = match SlabSize::pick_slab_size(layout.size()) {
            Some(SlabSize::Slab16) => &mut self.slab_16_bytes,
            Some(SlabSize::Slab32) => &mut self.slab_32_bytes,
            Some(SlabSize::Slab64) => &mut self.slab_64_bytes,
            Some(SlabSize::Slab128) => &mut self.slab_128_bytes,
            Some(SlabSize::Slab256) => &mut self.slab_256_bytes,
            Some(SlabSize::Slab512) => &mut self.slab_512_bytes,
            None => panic!("dealloc() called for unsupported block size."),
        };

        let node = ptr as *mut FreeListNode;
        slab.free_list.push(node);
    }
}

struct Slab {
    free_list: FreeList,
}

impl Slab {
    fn new(start: *mut u8, initial_size: usize, slab_block_size: SlabSize) -> Self {
        let slab_block_size = slab_block_size.len();
        let mut free_list = FreeList::new();

        let head = start as *mut FreeListNode;
        let mut current = head;

        unsafe {
            let mut ptr = start.offset(slab_block_size as isize);
            let end_ptr = start.offset(initial_size as isize);
            while ptr < end_ptr {
                let node = ptr as *mut FreeListNode;
                (*current).next = node;
                current = node;

                ptr = ptr.offset(slab_block_size as isize);
            }
        }

        free_list.head = head;

        Self { free_list }
    }
}

#[repr(C)]
struct FreeList {
    head: *mut FreeListNode,
}

impl FreeList {
    fn new() -> Self {
        Self {
            head: core::ptr::null_mut(),
        }
    }

    fn push(&mut self, new: *mut FreeListNode) {
        let head = self.head;
        unsafe {
            (*new).next = head;
        }
        self.head = new;
    }

    fn pop(&mut self) -> Option<*mut FreeListNode> {
        if self.head.is_null() {
            return None;
        }

        let result = self.head;
        unsafe {
            self.head = (*self.head).next;
        }
        Some(result)
    }
}

unsafe impl Send for FreeList {}
unsafe impl Sync for FreeList {}

struct FreeListNode {
    next: *mut FreeListNode,
}
