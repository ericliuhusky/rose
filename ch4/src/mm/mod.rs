//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;
mod elf_reader;

pub use address::{物理页, 将地址转为页号并向下取整, 虚拟页};
pub use memory_set::{MemorySet, KERNEL_SPACE};
pub use page_table::{PageTable, PageTableEntry};

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::FrameAllocator::init_frame_allocator();
    unsafe {
        KERNEL_SPACE = MemorySet::new_kernel();
        KERNEL_SPACE.activate();
    }
}
