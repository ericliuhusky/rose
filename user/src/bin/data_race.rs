#![no_std]
#![no_main]

#[macro_use]
extern crate lib;
extern crate alloc;

use alloc::vec::Vec;
use core::arch::asm;
use lib::{exit, sleep, thread_create, waittid};

#[no_mangle]
static mut A: usize = 0;

fn f() {
    unsafe {
        // A += 1;
        asm!(
            "
            la t0, A
            ld t1, 0(t0)
            addi t1, t1, 1
        "
        );
        sleep(10);
        asm!("sd t1, 0(t0)");
    }
    exit(0)
}

#[no_mangle]
pub fn main() -> i32 {
    let mut tids = Vec::new();
    for _ in 0..3 {
        tids.push(thread_create(f as usize, 0) as usize);
    }
    for tid in tids.into_iter() {
        waittid(tid);
    }
    println!("{}", unsafe { A });
    0
}
