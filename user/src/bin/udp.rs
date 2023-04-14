#![no_std]
#![no_main]

use alloc::string::String;

#[macro_use]
extern crate lib;
#[macro_use]
extern crate alloc;

use lib::{connect, read, write, socket, bind};

#[no_mangle]
pub fn main() -> i32 {
    println!("udp test open!");

    let udp_fd = socket(false);

    bind(udp_fd as usize, 2000);

    let mut buf = vec![0u8; 1024];

    let len = read(udp_fd as usize, &mut buf);

    let recv_str = String::from_utf8_lossy(&buf[..len as usize]);

    println!("receive reply <{}>", recv_str);

    let buf = "Hello rCoreOS user program!";

    println!("send <{}>", buf);

    write(udp_fd as usize, buf.as_bytes());

    println!("udp send done, waiting for reply.");

    0
}
