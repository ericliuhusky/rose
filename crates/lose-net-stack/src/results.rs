use crate::{packets::{udp::UDPPacket, tcp::TCPPacket}, net::Arp};

pub enum Packet {
    ARP(Arp),
    UDP(UDPPacket),
    TCP(TCPPacket<'static>),
    None
}

#[derive(Debug)]
pub enum NetStackErrors {
    NotRequiredReplyArp,
}