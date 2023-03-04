#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

#[no_mangle]
fn main() -> isize {
    println!("Into Test store_fault, we will insert an invalid store operation...");
    println!("Kernel should kill this application!");
    unsafe {
        let null_ptr = core::ptr::null_mut::<u8>();
        null_ptr.write_volatile(0);
    }
    0
}
