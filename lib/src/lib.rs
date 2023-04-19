#![no_std]
#![feature(alloc_error_handler)]

use core_ext::{println, print};
pub use sys_call::{exit, read, write, close, yield_, get_time, getpid, fork, exec, pipe, thread_create, mutex_create, mutex_lock, mutex_unlock, semaphore_create, semaphore_down, semaphore_up, connect, listen, accept, socket, bind};

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main();
    }
    // TODO: USER_HEAP_START_ADDR
    heap_allocator::init(0xFFFFFFFFFFF7F000, 0x80000);
    unsafe { main(); }
    exit();
}

pub fn open(path: &str, create: bool) -> isize {
    sys_call::open(path, create as usize)
}

pub fn getchar() -> u8 {
    sys_call::getchar() as u8
}

pub fn waitpid(pid: usize) {
    while sys_call::waitpid(pid) == -2 {
        yield_();
    }
}

pub fn sleep(period_ms: usize) {
    let start = get_time();
    while get_time() < start + period_ms as isize {
        yield_();
    }
}

pub fn waittid(tid: usize) {
    while sys_call::waittid(tid) == -2 {
        yield_();
    }
}
