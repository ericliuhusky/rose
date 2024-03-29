#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use lib::{fork, getpid, waitpid};
use lib::exit;

#[no_mangle]
pub fn main() {
    println!("[fork]");
    let pid = fork();
    if pid == 0 {
        println!("child[{}]", getpid());
        exit();
    }

    println!("parent[{}] waiting...", getpid());
    waitpid(pid);
    println!("child[{}] exit", pid);
}
