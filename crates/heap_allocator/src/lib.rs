#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

mod heap;
mod linked_list;
use heap::HeapAllocator;

pub fn init(start: usize, size: usize) {
    unsafe {
        HEAP_ALLOCATOR.init(start, size);
    }
}

#[global_allocator]
static mut HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
