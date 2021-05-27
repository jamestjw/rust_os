// The Bump allocator maintains a pointer that indicates
// the start address of the region it has available for
// allocation. Every time it allocates memory, it bumps
// this pointer up by the size required. The allocator
// also keeps track of the number of active allocations.
// The pointer will only ever be reset to the start of the
// heap when all allocations have been reclaimed.

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    // Initializes the allocator with the given heap bounds.
    //
    // This method is unsafe because it is up to the caller to
    // ensure that this memory range is unused. This method must
    // also not be called more than once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get a mutable reference
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());
        // Ensure no overflow on large allocations
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            // OOM
            ptr::null_mut()
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // get mutable reference

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
