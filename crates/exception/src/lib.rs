#![no_std]

pub mod context;
pub mod restore;
mod save;
#[cfg(feature = "memory_set")]
mod memory_set;

use riscv_register::stvec;

pub fn init() {
    extern "C" {
        fn __save();
    }    
    // 设置异常处理入口地址为__save
    stvec::write(__save as usize);
}
