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
        exit(0);
    }

    println!("parent[{}] waiting...", getpid());
    let mut exit_code: i32 = 0;
    let pid = wait(&mut exit_code);
    println!("child[{}] exit({})", pid, exit_code);
    0
}
