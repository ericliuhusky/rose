#![no_std]
#![no_main]

use core::arch::global_asm;
use sbi_call::shutdown;
use print::println;
use panic;
use entry;


#[no_mangle]
fn main() {
    println!("Hello, world!");
    shutdown();
}
