#![no_std]
#![feature(alloc_error_handler)]

use buddy_system_allocator::LockedHeap;

#[cfg(feature = "user")]
const HEAP_SIZE: usize = 0x4000;
#[cfg(not(feature = "user"))]
const HEAP_SIZE: usize = 0x200_0000;

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .0
            .borrow_mut()
            .init(&HEAP as *const [u8] as *const u8 as usize, HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
