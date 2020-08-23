//! RT-Thread allows static memory pool management and dynamic memory heap management. 
//! 
//! When static memory pool has available memory, the time allocated to the memory block will be constant; when the static memory pool is empty, the system will then request for suspending or blocking the thread of the memory block. (that is, the thread will abandon the request and return, if after waiting for a while, the memory block is not obtained or the thread will abandon and return immediately. The waiting time depends on the waiting time parameter set when the memory block is applied). When other threads release the memory block to the memory pool, if there is threads that are suspending and waiting to be allocated of memory blocks, the system will wake up the thread.
//! 
//! Under circumstances of different system resources, the dynamic memory heap management module respectively provides memory management algorithms for small memory systems and SLAB memory management algorithm for large memory systems.
//! 
//! There is also a dynamic memory heap management called memheap, which is suitable for memory heaps in systems with multiple addresses that can be discontinuous. Using memheap, you can "stick" multiple memory heaps together, letting the user operate as if he was operating a memory heap .

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
