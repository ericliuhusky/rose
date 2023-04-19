mod global;
mod linked_list;

use global::Global;

pub fn init(start: usize, size: usize) {
    unsafe {
        GLOBAL_ALLOCATOR.init(start, size);
    }
}

#[global_allocator]
static mut GLOBAL_ALLOCATOR: Global = Global::new();
