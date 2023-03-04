#![no_std]
#![feature(alloc_error_handler)]

use buddy_system_allocator::LockedHeap;

// sys 0x30_0000
const HEAP_SIZE: usize = 0x4000;

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(&HEAP as *const [u8] as *const u8 as usize, HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
