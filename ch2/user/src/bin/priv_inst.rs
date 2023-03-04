#![no_std]
#![no_main]

#[macro_use]
extern crate lib;
use core::arch::asm;

#[no_mangle]
fn main() -> isize {
    println!("Try to execute privileged instruction in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        asm!("sret");
    }
    0
}
