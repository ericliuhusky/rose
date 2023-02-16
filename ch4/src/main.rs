#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::arch::global_asm;

mod puts;
mod exit;
mod rust_no_std;
#[macro_use]
mod print;
mod syscall;
mod trap;
mod loader;
mod task;
mod config;
mod timer;
mod mm;

global_asm!(include_str!("entry.s"));
global_asm!(include_str!("link_app.s"));

#[no_mangle]
fn rust_main() {
    println!("[kernel] Hello, world!");
    mm::init();
    trap::init();
    timer::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
}
