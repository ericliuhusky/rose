#![no_std]
#![no_main]

#[macro_use]
extern crate lib;
extern crate alloc;

use alloc::vec;
use lib::{exit, semaphore_create, semaphore_down, semaphore_up, sleep, thread_create, waittid};

unsafe fn first() -> ! {
    sleep(10);
    println!("First work and wakeup Second");
    semaphore_up(0);
    exit()
}

unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait first");
    semaphore_down(0);
    println!("Second can work now");
    exit()
}

#[no_mangle]
pub fn main() -> i32 {
    semaphore_create(0);
    let threads = vec![
        thread_create(first as usize, 0),
        thread_create(second as usize, 0),
    ];
    for thread in threads.iter() {
        waittid(*thread);
    }
    0
}
