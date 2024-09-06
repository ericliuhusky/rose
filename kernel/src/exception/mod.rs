pub mod context;
mod exception_handler;
pub mod memory_set;
pub mod restore;
mod save;

use riscv::register::stvec;
use save::save;

pub fn init() {
    // 设置异常处理入口地址为save
    unsafe {
        stvec::write(save as usize, stvec::TrapMode::Direct);
    }
}

#[link_section = ".text.trampoline"]
static mut TRAP_CONTEXT_ADDR: usize = 0;
