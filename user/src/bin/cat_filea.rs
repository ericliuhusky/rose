#![no_std]
#![no_main]

#[macro_use]
extern crate lib;
extern crate alloc;

use lib::{close, open, read};

#[no_mangle]
pub fn main() {
    let fd = open("filea", false);
    if fd == 0 {
        panic!("Error occured when opening file");
    }
    let mut buf = [0u8; 256];
    let size = read(fd, &mut buf);
    println!("{}", core::str::from_utf8(&buf[..size]).unwrap());
    close(fd);
}
