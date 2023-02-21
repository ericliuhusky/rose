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
pub mod memory_set;
pub mod page_table;
mod elf_reader;

use memory_set::{MemorySet, KERNEL_SPACE};

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::初始化();
    frame_allocator::物理内存管理器::初始化();
    unsafe {
        KERNEL_SPACE = MemorySet::new_kernel();
        KERNEL_SPACE.activate();
    }
}
