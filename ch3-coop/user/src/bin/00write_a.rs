#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use user::yield_;

#[no_mangle]
fn main() -> i32 {
    for i in 1..=5 {
        println!("A [{}/{}]", i, 5);
        yield_::yield_();
    }
    0
}
