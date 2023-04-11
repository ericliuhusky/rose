#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use lib::{exec, fork, wait, yield_};

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec("shell");
    } else {
        loop {
            let pid = wait();
            if pid == -1 {
                yield_();
                continue;
            }
            println!(
                "[initproc] Released a zombie process, pid={}",
                pid,
            );
        }
    }
    0
}
