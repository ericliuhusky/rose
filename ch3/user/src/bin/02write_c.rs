#![no_std]
#![no_main]

#[macro_use]
extern crate user;

#[no_mangle]
fn main() -> i32 {
    for i in 1..=300 {
        println!("C [{}/{}]", i, 300);
    }
    0
}
