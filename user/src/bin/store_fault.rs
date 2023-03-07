#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

#[no_mangle]
fn main() {
    println!("null_ptr");
    unsafe {
        let null_ptr = core::ptr::null_mut::<u8>();
        null_ptr.write_volatile(0);
    }
}
