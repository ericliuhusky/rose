#![no_std]
#![no_main]

#[macro_use]
extern crate print;

mod syscall;
mod trap;
mod batch;
use core::arch::global_asm;
use panic;

global_asm!(include_str!("entry.s"));
global_asm!(include_str!("link_app.s"));

#[no_mangle]
fn rust_main() {
    println!("[kernel] Hello, world!");
    heap_allocator::init();
    trap::初始化();
    batch::应用管理器::初始化();
    batch::应用管理器::运行下一个应用();
}
