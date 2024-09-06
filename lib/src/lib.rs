#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

mod syscall;

pub use core_ext::{println, print, CoreExt};
use syscall::putchar;
pub use syscall::{exit, read, write, close, yield_, get_time, getpid, fork, exec, pipe, thread_create, mutex_create, mutex_lock, mutex_unlock, semaphore_create, semaphore_down, semaphore_up, listen, accept, socket, bind};
use alloc_ext::heap_alloc;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    core_ext::init(&CoreExtImpl);
    extern "C" {
        fn main();
    }
    // TODO: USER_HEAP_START_ADDR
    heap_alloc::init(0xFFFFFFFFFFF7F000, 0x80000);
    unsafe { main(); }
    exit();
}

struct CoreExtImpl;

impl CoreExt for CoreExtImpl {
    fn putchar(&self, c: char) {
        putchar(c as usize);
    }

    fn exit(&self) -> ! {
        exit();
    }
}

pub fn open(path: &str, create: bool) -> usize {
    syscall::open(path, create as usize)
}

pub fn getchar() -> u8 {
    syscall::getchar() as u8
}

pub fn waitpid(pid: usize) {
    while syscall::waitpid(pid) == 0 {
        yield_();
    }
}

pub fn sleep(period_ms: usize) {
    let start = get_time();
    while get_time() < start + period_ms {
        yield_();
    }
}

pub fn waittid(tid: usize) {
    while syscall::waittid(tid) == 0 {
        yield_();
    }
}
