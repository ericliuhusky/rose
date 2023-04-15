#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use core::str::from_utf8;
use lib::{accept, bind, close, listen, read, socket, write, connect};

#[no_mangle]
fn main() {
    let client_fd = socket(true);
    connect(client_fd as usize, 10 << 24 | 0 << 16 | 2 << 8 | 2, 80);

    let request = "Hello from client!";
    write(client_fd as usize, request.as_bytes());
    let mut buffer = [0u8; 1024];
    read(client_fd as usize, &mut buffer);
    println!("{}", from_utf8(&buffer).unwrap());
    close(client_fd as usize);
}
