#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use user::{fork, getpid, wait};
use user::终止::终止;

#[no_mangle]
pub fn main() -> i32 {
    println!("[fork]");
    if fork() == 0 {
        println!("child[{}]", getpid());
        终止(0);
    }

    println!("parent[{}] waiting...", getpid());
    let mut exit_code: i32 = 0;
    let pid = wait(&mut exit_code);
    println!("child[{}] exit({})", pid, exit_code);
    0
}
