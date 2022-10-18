#![no_std]
#![no_main]

#[macro_use]
extern crate user;

#[no_mangle]
fn main() -> i32 {
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        core::arch::asm!("csrw sstatus, {}", in(reg) 1 << 8);
    }
    0
}
