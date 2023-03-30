pub mod memory_set;

use exception::{memory_set::{switch_kernel, set_kernel_satp}, set_kernel_top};
use memory_set::KERNEL_SPACE;
use riscv_register::satp;

use crate::mm::memory_set::{MEMORY_END, KERNEL_STACK_TOP};

pub fn 初始化() {
    static mut HEAP: [u8; 0x4000] = [0; 0x4000];
    heap_allocator::init(
        unsafe { &HEAP } as *const [u8] as *const u8 as usize,
        0x4000,
    );
    frame_allocator::init(MEMORY_END);
    set_kernel_top(KERNEL_STACK_TOP);
    unsafe {
        memory_set::init();
        // 切换到内核地址空间
        let satp = KERNEL_SPACE.as_ref().unwrap().token();
        set_kernel_satp(satp);
        switch_kernel();
    }
}
