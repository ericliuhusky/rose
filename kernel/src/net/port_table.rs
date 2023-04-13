use alloc::vec::Vec;
use core::cell::RefCell;
use lazy_static::lazy_static;
use lose_net_stack::packets::tcp::TCPPacket;
use mutrc::MutRc;

use crate::fs::File;
use crate::task::current_task;

use super::tcp::TCP;

pub struct Port {
    pub port: u16,
}

lazy_static! {
    static ref LISTEN_TABLE: RefCell<Vec<MutRc<Port>>> = RefCell::new(Vec::new());
}

pub fn listen(port: u16) -> MutRc<Port> {
    let mut listen_table = LISTEN_TABLE.borrow_mut();

    let listen_port = MutRc::new(Port {
        port,
    });

    listen_table.push(listen_port.clone());
    listen_port
}

// check whether it can accept request
pub fn check_accept(port: u16, tcp_packet: &TCPPacket) -> Option<TCP> {
    let mut listen_table = LISTEN_TABLE.borrow_mut();
    let listen_port = listen_table.iter_mut().find(|p| p.port == port);
    if let Some(listen_port) = listen_port {

        Some(accept_connection(port, tcp_packet))
    } else {
        None
    }
}

pub fn accept_connection(_port: u16, tcp_packet: &TCPPacket) -> TCP {
    let tcp_socket = TCP::new(
        tcp_packet.source_ip,
        tcp_packet.dest_port,
        tcp_packet.source_port,
        tcp_packet.seq,
        tcp_packet.ack,
    );
    tcp_socket
}

impl File for Port {
    fn read(&mut self, _buf: Vec<&'static mut [u8]>) -> usize {
        unimplemented!()
    }

    fn write(&mut self, _buf: Vec<&'static mut [u8]>) -> usize {
        unimplemented!()
    }
}
