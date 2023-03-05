#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use lib::{get_time, sleep};

#[no_mangle]
pub fn main() -> i32 {
    println!("[sleep]");
    println!("start time: {}", get_time());
    sleep(1000);
    println!("end time: {}", get_time());
    0
}
