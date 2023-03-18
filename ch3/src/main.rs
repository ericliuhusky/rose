#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate print;

mod syscall;
mod task;
mod timer;
mod segment;
mod exception_handler;
use core::arch::global_asm;
extern crate panic;
extern crate entry;

global_asm!(include_str!("link_app.s"));

#[no_mangle]
fn main() {
    println!("[kernel] Hello, world!");
    static mut HEAP: [u8; 0x4000] = [0; 0x4000];
    heap_allocator::init(
        unsafe { &HEAP } as *const [u8] as *const u8 as usize,
        0x4000,
    );
    exception::init();
    segment::init();
    timer::开启时钟中断();
    timer::为下一次时钟中断定时();
    task::init();
    task::run_next();
}
