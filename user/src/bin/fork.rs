#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use lib::{fork, getpid, wait};
use lib::exit;

#[no_mangle]
pub fn main() -> i32 {
    println!("[fork]");
    if fork() == 0 {
        println!("child[{}]", getpid());
        exit();
    }

    println!("parent[{}] waiting...", getpid());
    let pid = wait();
    println!("child[{}] exit", pid);
    0
}
