pub mod memory_set;

use crate::exception::{
    memory_set::{set_kernel_satp, switch_kernel},
};
use memory_set::KERNEL_SPACE;

use crate::mm::memory_set::{KERNEL_HEAP_SIZE, KERNEL_HEAP_START_ADDR, KERNEL_STACK_TOP};

use self::memory_set::AVAILABLE_MEMORY_END;
use alloc_ext::heap_alloc;

pub fn 初始化() {
    heap_alloc::init(KERNEL_HEAP_START_ADDR, KERNEL_HEAP_SIZE);
    frame_allocator::init(AVAILABLE_MEMORY_END);
    // 切换到内核地址空间
    let satp = KERNEL_SPACE.page_table.satp();
    set_kernel_satp(satp);
    switch_kernel();
}
