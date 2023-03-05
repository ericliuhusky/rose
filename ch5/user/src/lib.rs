#![no_std]
#![feature(alloc_error_handler)]

use print::{println, print};
pub use sys_call::exit;
use panic;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main() -> isize;
    }
    heap_allocator::init();
    let 终止代码 = unsafe { main() };
    exit(终止代码);
}

// 一个耗时程序，用以验证10ms之后会自动切换下一个任务
pub fn fibonacci(x: u32) -> u32 {
    if x == 0 { return 0 }
    if x == 1 { return 1 }
    fibonacci(x - 2) + fibonacci(x - 1)
}

use core::arch::asm;

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    read(&mut c);
    c[0]
}

pub use sys_call::{read, yield_, get_time, getpid, fork, exec};

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_call::waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_call::waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
pub fn sleep(period_ms: usize) {
    let start = get_time();
    while get_time() < start + period_ms as isize {
        yield_();
    }
}
