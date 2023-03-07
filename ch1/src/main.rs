#![no_std]
#![no_main]

use sbi_call::shutdown;
use print::println;
extern crate panic;
extern crate entry;


#[no_mangle]
fn main() {
    println!("Hello, world!");
    shutdown();
}
