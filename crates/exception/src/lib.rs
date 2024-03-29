#![no_std]
#![feature(naked_functions)]
#![feature(fn_align)]

pub mod context;
pub mod restore;
mod save;
pub mod memory_set;

use riscv_register::stvec;
use save::save;

pub fn init() {
    // 设置异常处理入口地址为save
    stvec::write(save as usize);
}

#[no_mangle]
#[link_section = ".text.trampoline"]
static mut KERNEL_STACK_TOP: usize = 0;

pub fn set_kernel_top(addr: usize) {
    unsafe {
        KERNEL_STACK_TOP = addr;
    }
}

#[no_mangle]
#[link_section = ".text.trampoline"]
static mut TRAP_CONTEXT_ADDR: usize = 0;
