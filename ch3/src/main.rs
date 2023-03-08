#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate print;

mod syscall;
mod trap;
mod loader;
mod task;
mod timer;
use core::arch::global_asm;
extern crate panic;
extern crate entry;

global_asm!(include_str!("link_app.s"));

#[no_mangle]
fn main() {
    println!("[kernel] Hello, world!");
    heap_allocator::init();
    trap::初始化();
    timer::开启时钟中断();
    timer::为下一次时钟中断定时();
    loader::init();
    task::任务管理器::初始化();
    task::任务管理器::运行下一个任务();
}
