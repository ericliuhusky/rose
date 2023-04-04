#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate print;


mod syscall;
mod exception_handler;
mod task;
mod timer;
mod mm;
mod drivers;
mod fs;
mod mutrc;

use core::arch::global_asm;
extern crate panic;
extern crate entry;

global_asm!(include_str!("link_app.s"));

#[no_mangle]
fn main() {
    println!("[kernel] Hello, world!");
    mm::初始化();
    exception::init();
    timer::开启时钟中断();
    timer::为下一次时钟中断定时();
    task::add_initproc();
    task::run_next();
}