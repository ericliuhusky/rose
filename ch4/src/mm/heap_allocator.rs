use buddy_system_allocator::LockedHeap;

static mut 内核堆: [u8; 0x30_0000] = [0; 0x30_0000];

#[global_allocator]
static 堆管理器: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

pub fn 初始化() {
    unsafe {
        堆管理器
            .lock()
            .init(内核堆.as_ptr() as usize, 内核堆.len());
    }
}
