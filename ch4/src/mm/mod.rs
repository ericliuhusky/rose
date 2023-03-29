pub mod memory_set;

use exception::memory_set::switch_kernel;
use memory_set::KERNEL_SPACE;
use riscv_register::satp;

use crate::mm::memory_set::MEMORY_END;

#[no_mangle]
#[link_section = ".text.trampoline"]
static mut KERNEL_SATP: usize = 0;
#[no_mangle]
#[link_section = ".text.trampoline"]
pub static mut USER_SATP: usize = 0;

pub fn 初始化() {
    static mut HEAP: [u8; 0x4000] = [0; 0x4000];
    heap_allocator::init(
        unsafe { &HEAP } as *const [u8] as *const u8 as usize,
        0x4000,
    );
    frame_allocator::init(MEMORY_END);
    unsafe {
        memory_set::init();
        // 切换到内核地址空间
        let satp = KERNEL_SPACE.as_ref().unwrap().token();
        KERNEL_SATP = satp;
        switch_kernel();
    }
}
