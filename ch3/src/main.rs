#![no_std]
#![no_main]

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

global_asm!(include_str!("entry.s"));
global_asm!(include_str!("link_app.s"));

#[no_mangle]
fn rust_main() {
    println!("[kernel] Hello, world!");
    trap::init();
    loader::load_apps();
    task::run_first_task();
}
