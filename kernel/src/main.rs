#![no_std]
#![no_main]
#![feature(linkage, naked_functions, fn_align)]

extern crate alloc;
#[macro_use]
extern crate core_ext;
#[macro_use]
extern crate bitflags;

mod drivers;
mod exception;
mod fs;
mod mm;
mod mutex;
mod net;
mod semaphore;
mod syscall;
mod task;
mod timer;

use core::arch::asm;
use core_ext::CoreExt;

const KERNEL_STACK_TOP: usize = 0x87800000;

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    asm!(
        "
        li sp, {KERNEL_STACK_TOP}
        call {main}
        ",
        KERNEL_STACK_TOP = const KERNEL_STACK_TOP,
        main = sym main,
        options(noreturn)
    );
}

fn main() {
    core_ext::init(&CoreExtImpl);
    println!("[kernel] Hello, world!");
    drivers::init();
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
