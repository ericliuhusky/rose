mod address;
mod frame_allocator;
pub mod memory_set;

use memory_set::内核地址空间;
use riscv_register::satp;

pub fn 初始化() {
    heap::init();
    frame_allocator::物理内存管理器::初始化();
    unsafe {
        // 切换到内核地址空间
        let satp = 内核地址空间.token();
        satp::write(satp);
        core::arch::asm!("sfence.vma");
    }
}
