use std::{net::TcpStream, io::{Write, Read}, str::from_utf8};

fn main() {
    let mut tcp = TcpStream::connect("127.0.0.1:3000").unwrap();
    tcp.write("Hello, server!".as_bytes()).unwrap();
    let mut buf = [0; 1024];
    tcp.read(&mut buf).unwrap();
    println!("{}", from_utf8(&buf).unwrap());
}
