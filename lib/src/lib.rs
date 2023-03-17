#![no_std]
#![feature(alloc_error_handler)]

use print::{println, print};
pub use sys_call::{exit, read, write, close, yield_, get_time, getpid, fork, exec};
extern crate panic;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main();
    }
    static mut HEAP: [u8; 0x4000] = [0; 0x4000];
    heap_allocator::init(
        unsafe { &HEAP } as *const [u8] as *const u8 as usize,
        0x4000,
    );
    unsafe { main(); }
    exit(0);
}

pub fn open(path: &str, create: bool) -> isize {
    sys_call::open(path, create as usize)
}

pub fn getchar() -> u8 {
    sys_call::getchar() as u8
}

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
