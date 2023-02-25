mod address;
mod frame_allocator;
pub mod memory_set;
pub mod page_table;
mod elf_reader;

use memory_set::{地址空间, 内核地址空间};

pub fn 初始化() {
    堆::初始化();
    frame_allocator::物理内存管理器::初始化();
    unsafe {
        内核地址空间 = 地址空间::新建内核地址空间();
        内核地址空间.切换到当前地址空间();
    }
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
