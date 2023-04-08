#![no_std]
#![no_main]

#[macro_use]
extern crate lib;
extern crate alloc;

use alloc::vec;
use lib::{exit, thread_create, waittid, sleep};

pub fn thread_a() -> ! {
    for i in 0..3 {
        sleep(10);
        println!("A {}", i);
    }
    exit(1)
}

pub fn thread_b() -> ! {
    for i in 0..3 {
        sleep(10);
        println!("B {}", i);
    }
    exit(2)
}

pub fn thread_c() -> ! {
    for i in 0..3 {
        sleep(10);
        println!("C {}", i);
    }
    exit(3)
}

#[no_mangle]
pub fn main() -> i32 {
    let v = vec![
        thread_create(thread_a as usize, 0),
        thread_create(thread_b as usize, 0),
        thread_create(thread_c as usize, 0),
    ];
    for tid in v.iter() {
        let exit_code = waittid(*tid as usize);
        println!("thread[{}] exit({})", tid, exit_code);
    }
    println!("main thread exited.");
    0
}
