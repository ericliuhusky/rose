use alloc::{vec::Vec, collections::BTreeSet};
use core::cell::RefCell;
use lazy_static::lazy_static;
use lose_net_stack::packets::tcp::TCPPacket;
use mutrc::MutRc;

use crate::fs::File;

use super::tcp::TCP;

pub struct Port {
    pub port: u16,
}

lazy_static! {
    static ref LISTEN_PORTS: RefCell<BTreeSet<u16>> = RefCell::new(BTreeSet::new());
}

pub fn listen(port: u16) -> MutRc<Port> {
    let mut listen_table = LISTEN_PORTS.borrow_mut();
    listen_table.insert(port);

    let listen_port = MutRc::new(Port {
        port,
    });
    listen_port
}

// check whether it can accept request
pub fn check_accept(port: u16, tcp_packet: &TCPPacket) -> Option<TCP> {
    let listen_table = LISTEN_PORTS.borrow();
    if listen_table.contains(&port) {
        Some(TCP::new(
            tcp_packet.source_ip,
            tcp_packet.dest_port,
            tcp_packet.source_port,
            tcp_packet.seq,
            tcp_packet.ack,
        ))
    } else {
        None
    }
}

impl File for Port {
    fn read(&mut self, _buf: Vec<&'static mut [u8]>) -> usize {
        unimplemented!()
    }

    fn write(&mut self, _buf: Vec<&'static mut [u8]>) -> usize {
        unimplemented!()
    }
}
