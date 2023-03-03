mod frame_allocator;
pub mod memory_set;

use memory_set::内核地址空间;

pub fn 初始化() {
    堆::初始化();
    frame_allocator::物理内存管理器::初始化();
    unsafe {
        // 切换到内核地址空间
        let satp = 内核地址空间.token();
        core::arch::asm!("csrw satp, {}", in(reg) satp);
        core::arch::asm!("sfence.vma");
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
