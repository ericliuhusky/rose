#![no_std]
#![no_main]

#[macro_use]
extern crate lib;
use core::arch::asm;

#[no_mangle]
fn main() {
    println!("user sret");
    unsafe {
        asm!("sret");
    }
}
