#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use core::str::from_utf8;
use lib::{accept, bind, close, listen, read, socket, write};

#[no_mangle]
fn main() {
    let server_fd = socket(true);
    bind(server_fd as usize, 80);
    listen(server_fd as usize);

    println!("Server listening...");

    loop {
        let client_fd = accept(server_fd as usize);
        let mut buffer = [0u8; 1024];
        read(client_fd as usize, &mut buffer);
        println!("{}", from_utf8(&buffer).unwrap());
        let response = "Hello from server!";
        write(client_fd as usize, response.as_bytes());
        close(client_fd as usize);
    }
}
