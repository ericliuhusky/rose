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
    static mut HEAP: [u8; 0x4000] = [0; 0x4000];
    heap_allocator::init(
        unsafe { &HEAP } as *const [u8] as *const u8 as usize,
        0x4000,
    );
    exception::init();
    segment::init();
    batch::应用管理器::初始化();
    batch::应用管理器::运行下一个应用();
}
