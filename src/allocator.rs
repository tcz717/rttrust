use crate::ffi::{rt_free_align, rt_malloc_align, rt_size_t};
use core::alloc::GlobalAlloc;

#[global_allocator]
static ALLOCATOR: SystemHeapAllocator = SystemHeapAllocator;

pub struct SystemHeapAllocator;

unsafe impl GlobalAlloc for SystemHeapAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        rt_malloc_align(layout.size() as rt_size_t, layout.align() as rt_size_t).cast()
    }
    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        rt_free_align(ptr.cast())
    }
}
