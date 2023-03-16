#![no_std]
#![no_main]

#[macro_use]
extern crate lib;
extern crate alloc;

use lib::{close, open, read};

#[no_mangle]
pub fn main() -> i32 {
    let fd = open("filea", false);
    if fd == -1 {
        panic!("Error occured when opening file");
    }
    let fd = fd as usize;
    let mut buf = [0u8; 256];
    loop {
        let size = read(fd, &mut buf) as usize;
        if size == 0 {
            break;
        }
        println!("{}", core::str::from_utf8(&buf[..size]).unwrap());
    }
    close(fd);
    0
}
