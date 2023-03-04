#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

#[no_mangle]
fn main() -> i32 {
    println!("Into Test store_fault, we will insert an invalid store operation...");
    println!("Kernel should kill this application!");
    unsafe {
        let 空指针 = core::ptr::null_mut::<u8>();
        空指针.write_volatile(0);
    }
    0
}
