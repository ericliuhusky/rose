#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate core_ext;
#[macro_use]
extern crate bitflags;
extern crate entry;

mod drivers;
mod exception_handler;
mod fs;
mod mm;
mod mutex;
mod net;
mod semaphore;
mod syscall;
mod task;
mod timer;

use core_ext::CoreExt;

#[no_mangle]
fn main() {
    core_ext::init(&CoreExtImpl);
    println!("[kernel] Hello, world!");
    mm::初始化();
    exception::init();
    timer::开启时钟中断();
    timer::为下一次时钟中断定时();
    task::add_initproc();
    task::run_next();
}

struct CoreExtImpl;

impl CoreExt for CoreExtImpl {
    fn putchar(&self, c: char) {
        sbi_call::putchar(c as usize);
    }

    fn exit(&self) -> ! {
        sbi_call::shutdown();
    }
}
