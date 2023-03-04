#![no_std]
#![no_main]

#[macro_use]
extern crate user;
use user::fibonacci;

#[no_mangle]
fn main() -> i32 {
    for i in 1..=30 {
        let r = fibonacci(i);
        println!("A [{}/{}] {}", i, 30, r);
    }
    0
}
