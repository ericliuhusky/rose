#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use lib::{close, open, read, write};

#[no_mangle]
pub fn main() {
    let test_str = "Hello, world!";
    let filea = "filea";
    let fd = open(filea, true);
    assert!(fd > 0);
    write(fd, test_str.as_bytes());
    close(fd);

    let fd = open(filea, false);
    assert!(fd > 0);
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer);
    close(fd);

    assert_eq!(test_str, core::str::from_utf8(&buffer[..read_len]).unwrap(),);
    println!("file_test passed!");
}
