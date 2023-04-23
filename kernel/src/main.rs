#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate core_ext;
#[macro_use]
extern crate bitflags;
extern crate entry;

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

use alloc::boxed::Box;
use core_ext::{CoreExt, CORE_EXT};

#[no_mangle]
fn main() {
    unsafe {
        CORE_EXT = Some(Box::new(CoreExtImpl));
    }
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
