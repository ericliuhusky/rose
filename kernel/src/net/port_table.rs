use super::tcp::TCP;
use alloc::collections::BTreeSet;
use core::cell::RefCell;
use lazy_static::lazy_static;
use lose_net_stack::packets::tcp::TCPPacket;

lazy_static! {
    static ref LISTEN_PORTS: RefCell<BTreeSet<u16>> = RefCell::new(BTreeSet::new());
}

pub fn listen(port: u16) {
    let mut listen_table = LISTEN_PORTS.borrow_mut();
    listen_table.insert(port);
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
