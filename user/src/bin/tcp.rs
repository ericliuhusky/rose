#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use core::str::from_utf8;
use lib::{accept, bind, close, listen, read, socket, write};

#[no_mangle]
fn main() {
    let server_fd = socket(true);
    bind(server_fd, 80);
    listen(server_fd);

    println!("Server listening...");

    loop {
        let client_fd = accept(server_fd);
        let mut buffer = [0u8; 1024];
        read(client_fd, &mut buffer);
        println!("{}", from_utf8(&buffer).unwrap());
        let response = "Hello from server!";
        write(client_fd, response.as_bytes());
        close(client_fd);
    }
}
