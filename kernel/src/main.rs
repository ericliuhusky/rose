#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate print;
#[macro_use]
extern crate bitflags;

mod syscall;
mod exception_handler;
mod task;
mod timer;
mod mm;
mod drivers;
mod fs;
mod mutex;
mod semaphore;
mod net;

extern crate panic;
extern crate entry;

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
