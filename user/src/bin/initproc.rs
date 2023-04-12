#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use lib::{exec, fork, waitpid, yield_};

#[no_mangle]
fn main() -> i32 {
    let pid = fork();
    if pid == 0 {
        exec("shell");
    } else {
        loop {
            waitpid(pid as usize);
            println!(
                "[initproc] Released a zombie process, pid={}",
                pid,
            );
        }
    }
    0
}
