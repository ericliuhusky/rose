use std::{net::UdpSocket, str::from_utf8};

fn main() {
    let udp = UdpSocket::bind("0.0.0.0:0").unwrap();
    udp.connect("127.0.0.1:6000").unwrap();
    udp.send("Hello, server!".as_bytes()).unwrap();
    let mut buf = [0; 1024];
    udp.recv(&mut buf).unwrap();
    println!("{}", from_utf8(&buf).unwrap());
}
