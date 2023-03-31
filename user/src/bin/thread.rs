#![no_std]
#![no_main]

#[macro_use]
extern crate lib;
extern crate alloc;

use alloc::vec;
use lib::{exit, thread_create, waittid};

pub fn thread_a() -> ! {
    for i in 26..=33 {
        let r = fibonacci(i);
        println!("A[{}] {}", i, r);
    }
    exit(1)
}

pub fn thread_b() -> ! {
    for i in 26..=33 {
        let r = fibonacci(i);
        println!("B[{}] {}", i, r);
    }
    exit(2)
}

pub fn thread_c() -> ! {
    for i in 26..=33 {
        let r = fibonacci(i);
        println!("C[{}] {}", i, r);
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

// 一个耗时程序，用以验证10ms之后会自动切换下一个任务
pub fn fibonacci(x: u32) -> u32 {
    if x == 0 { return 0 }
    if x == 1 { return 1 }
    fibonacci(x - 2) + fibonacci(x - 1)
}
