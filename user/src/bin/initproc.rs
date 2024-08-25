#![no_std]
#![no_main]

extern crate lib;

use lib::{exec, fork, waitpid};

#[no_mangle]
fn main() {
    let pid = fork();
    if pid == 0 {
        exec("shell");
    } else {
        waitpid(pid);
    }
}
