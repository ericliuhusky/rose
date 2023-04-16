use crate::{packets::{udp::UDPPacket, tcp::TCPPacket}, net::Arp};

pub enum Packet {
    ARP(Arp),
    UDP(UDPPacket<'static>),
    TCP(TCPPacket<'static>),
    None
}

#[derive(Debug)]
pub enum NetStackErrors {
    NotRequiredReplyArp,
}