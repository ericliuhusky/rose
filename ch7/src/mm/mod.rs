mod address;
pub mod memory_set;

use exception::{memory_set::{switch_kernel, set_kernel_satp}, set_kernel_top};
use memory_set::内核地址空间;
use riscv_register::satp;

use crate::mm::memory_set::KERNEL_STACK_TOP;

use self::memory_set::MEMORY_END;

pub fn 初始化() {
    static mut HEAP: [u8; 0x800000] = [0; 0x800000];
    heap_allocator::init(
        unsafe { &HEAP } as *const [u8] as *const u8 as usize,
        0x800000,
    );
    frame_allocator::init(MEMORY_END);
    set_kernel_top(KERNEL_STACK_TOP);
    unsafe {
        // 切换到内核地址空间
        let satp = 内核地址空间.token();
        set_kernel_satp(satp);
        switch_kernel();
    }
}
