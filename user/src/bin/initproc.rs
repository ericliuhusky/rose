#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use lib::{exec, fork, waitpid};

#[no_mangle]
fn main() -> i32 {
    let pid = fork();
    if pid == 0 {
        exec("shell");
    } else {
        waitpid(pid);
    }
    0
}
