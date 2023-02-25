//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
pub mod memory_set;
pub mod page_table;
mod elf_reader;

use memory_set::内核地址空间;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    堆::初始化();
    frame_allocator::物理内存管理器::初始化();
    内核地址空间::初始化();
}

mod 堆 {
    use buddy_system_allocator::LockedHeap;

    static mut 内核堆: [u8; 0x30_0000] = [0; 0x30_0000];

    #[global_allocator]
    static 堆管理器: LockedHeap = LockedHeap::empty();

    pub fn 初始化() {
        unsafe {
            堆管理器
                .lock()
                .init(内核堆.as_ptr() as usize, 内核堆.len());
        }
    }
}
