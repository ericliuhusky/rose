mod frame_allocator;
pub mod memory_set;

use memory_set::内核地址空间;
use riscv_register::satp;

#[no_mangle]
#[link_section = ".text.trampoline"]
static mut KERNEL_SATP: usize = 0;
#[no_mangle]
#[link_section = ".text.trampoline"]
pub static mut USER_SATP: usize = 0;

pub fn 初始化() {
    heap_allocator::init();
    frame_allocator::物理内存管理器::初始化();
    unsafe {
        // 切换到内核地址空间
        let satp = 内核地址空间.token();
        KERNEL_SATP = satp;
        satp::write(satp);
        core::arch::asm!("sfence.vma");
    }
}
