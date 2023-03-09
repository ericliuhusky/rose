#![no_std]
#![no_main]

#[macro_use]
extern crate print;

mod syscall;
mod exception_handler;
mod batch;
mod segment;
use core::arch::global_asm;
extern crate panic;
extern crate entry;

global_asm!(include_str!("link_app.s"));

#[no_mangle]
fn main() {
    heap_allocator::init();
    exception_handler::初始化();
    segment::init();
    batch::应用管理器::初始化();
    batch::应用管理器::运行下一个应用();
}
